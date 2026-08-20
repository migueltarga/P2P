//! Peer que descobre o próprio endpoint com STUN, se apresenta pelo
//! rendezvous e abre um caminho UDP direto com hole punching.

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket as StdUdpSocket};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use stunclient::StunClient;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};

const STUN_SERVER: &str = "stun.cloudflare.com:3478";
const PUNCH_INTERVAL: Duration = Duration::from_millis(350);
const PUNCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Serialize, Deserialize)]
struct Registration {
    room: String,
    peer_id: String,
    candidates: Vec<SocketAddr>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let [_, peer_id, room, ..] = arguments.as_slice() else {
        eprintln!("uso: peer PEER_ID SALA [RENDEZVOUS]");
        return Ok(());
    };
    let rendezvous = arguments
        .get(3)
        .cloned()
        .unwrap_or_else(|| "127.0.0.1:7777".to_string());

    // Um socket, do começo ao fim: o mesmo que consulta STUN e conversa.
    let socket = StdUdpSocket::bind("0.0.0.0:0")?;
    let candidates = gather_candidates(&socket)?;
    println!("candidates locais:");
    for candidate in &candidates {
        println!("  {candidate}");
    }

    socket.set_nonblocking(true)?;
    let socket = UdpSocket::from_std(socket)?;

    let me = Registration {
        room: room.clone(),
        peer_id: peer_id.clone(),
        candidates,
    };
    let other = introduce(&rendezvous, &me).await?;
    println!("candidates de {}:", other.peer_id);
    for candidate in &other.candidates {
        println!("  {candidate}");
    }
    println!("tentando abrir um caminho; /quit encerra");

    punch_and_chat(socket, other).await
}

/// Lista os endereços por onde o outro peer pode tentar chegar.
fn gather_candidates(socket: &StdUdpSocket) -> Result<Vec<SocketAddr>, Box<dyn Error>> {
    let port = socket.local_addr()?.port();
    let mut candidates = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)];
    if let Some(address) = routed_local_ip() {
        candidates.push(SocketAddr::new(address, port));
    }

    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let server = STUN_SERVER
        .to_socket_addrs()?
        .next()
        .ok_or("não foi possível resolver o servidor STUN")?;
    let reflexive = StunClient::new(server).query_external_address(socket)?;
    if !candidates.contains(&reflexive) {
        candidates.push(reflexive);
    }

    Ok(candidates)
}

/// Descobre o IP da interface que o sistema usa para alcançar a internet.
fn routed_local_ip() -> Option<IpAddr> {
    let probe = StdUdpSocket::bind("0.0.0.0:0").ok()?;
    probe.connect("1.1.1.1:80").ok()?;
    Some(probe.local_addr().ok()?.ip())
}

/// Registra no rendezvous e espera a linha com os dados do outro peer.
async fn introduce(address: &str, me: &Registration) -> Result<Registration, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address).await?;
    let mut payload = serde_json::to_string(me)?;
    payload.push('\n');
    stream.write_all(payload.as_bytes()).await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).await? == 0 {
        return Err("o rendezvous fechou a conexão sem apresentar ninguém".into());
    }
    Ok(serde_json::from_str(&line)?)
}

async fn punch_and_chat(socket: UdpSocket, other: Registration) -> Result<(), Box<dyn Error>> {
    let mut endpoint: Option<SocketAddr> = None;
    let mut buffer = [0_u8; 1500];
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut punch = tokio::time::interval(PUNCH_INTERVAL);
    let timeout = tokio::time::sleep(PUNCH_TIMEOUT);
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            // Os dois lados enviam primeiro: é isso que abre os NATs.
            _ = punch.tick(), if endpoint.is_none() => {
                for candidate in &other.candidates {
                    socket.send_to(b"HELLO", candidate).await?;
                }
            }
            _ = &mut timeout, if endpoint.is_none() => {
                eprintln!("nenhum candidate respondeu; não há caminho direto");
                return Ok(());
            }
            received = socket.recv_from(&mut buffer) => {
                let (size, from) = received?;
                match &buffer[..size] {
                    b"HELLO" => {
                        socket.send_to(b"ACK", from).await?;
                        connect(&mut endpoint, from, &other.peer_id);
                    }
                    b"ACK" => connect(&mut endpoint, from, &other.peer_id),
                    b"BYE" if endpoint == Some(from) => {
                        println!("{} desconectou", other.peer_id);
                        return Ok(());
                    }
                    message if endpoint == Some(from) => {
                        println!("[{}] {}", other.peer_id, String::from_utf8_lossy(message));
                    }
                    _ => {}
                }
            }
            line = lines.next_line() => {
                match line? {
                    None => break,
                    Some(text) if text == "/quit" => break,
                    Some(text) if text.is_empty() => {}
                    Some(text) => match endpoint {
                        Some(address) => { socket.send_to(text.as_bytes(), address).await?; }
                        None => eprintln!("ainda não há caminho aberto"),
                    },
                }
            }
        }
    }

    if let Some(address) = endpoint {
        socket.send_to(b"BYE", address).await?;
    }
    Ok(())
}

/// A origem da primeira resposta válida vira o endpoint da sessão.
fn connect(endpoint: &mut Option<SocketAddr>, from: SocketAddr, peer_id: &str) {
    if endpoint.is_none() {
        println!("conectado a {peer_id} por {from}");
        *endpoint = Some(from);
    }
}
