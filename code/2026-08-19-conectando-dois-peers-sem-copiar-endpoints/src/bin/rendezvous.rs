//! Rendezvous mínimo: apresenta dois peers de uma sala e sai do caminho.
//!
//! Cada peer abre uma conexão TCP, envia uma linha JSON com identidade e
//! endereços candidatos, e recebe de volta a linha do outro peer. Depois disso o
//! servidor fecha a conexão: ele não transporta a conversa.

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

#[derive(Clone, Serialize, Deserialize)]
struct Registration {
    room: String,
    peer_id: String,
    candidates: Vec<SocketAddr>,
}

/// Cada sala guarda no máximo um peer esperando: o registro dele e o canal
/// por onde o segundo a chegar será entregue.
type Rooms = Arc<Mutex<HashMap<String, (Registration, oneshot::Sender<Registration>)>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:7777".to_string());
    let listener = TcpListener::bind(&address).await?;
    println!("rendezvous escutando em {}", listener.local_addr()?);

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, from) = listener.accept().await?;
        let rooms = Arc::clone(&rooms);
        tokio::spawn(async move {
            if let Err(error) = introduce(stream, rooms).await {
                eprintln!("{from}: {error}");
            }
        });
    }
}

async fn introduce(stream: TcpStream, rooms: Rooms) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let me: Registration = serde_json::from_str(&line)?;

    // Quem chega primeiro deixa um canal na sala e espera. Quem chega depois
    // encontra esse canal, entrega o próprio registro e leva o do primeiro.
    let (notify, wait) = oneshot::channel();
    let paired = {
        let mut rooms = rooms.lock().unwrap();
        match rooms.remove(&me.room) {
            Some((first, first_notify)) if first.peer_id == me.peer_id => {
                rooms.insert(me.room, (first, first_notify));
                return Err("id repetido na sala".into());
            }
            Some((first, first_notify)) => {
                let _ = first_notify.send(me.clone());
                Some(first)
            }
            None => {
                rooms.insert(me.room.clone(), (me.clone(), notify));
                None
            }
        }
    };

    let other = match paired {
        Some(first) => {
            println!("sala {}: {} e {}", me.room, first.peer_id, me.peer_id);
            first
        }
        None => {
            println!("sala {}: {} aguardando", me.room, me.peer_id);
            // Se o peer desistir antes do par chegar, a sala volta a ficar livre.
            let mut discarded = String::new();
            tokio::select! {
                second = wait => second?,
                result = reader.read_line(&mut discarded) => {
                    rooms.lock().unwrap().remove(&me.room);
                    result?;
                    return Err("desconectou antes do par chegar".into());
                }
            }
        }
    };

    let mut payload = serde_json::to_string(&other)?;
    payload.push('\n');
    reader.get_mut().write_all(payload.as_bytes()).await?;
    Ok(())
}
