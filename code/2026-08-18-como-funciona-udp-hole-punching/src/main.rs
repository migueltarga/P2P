use std::env;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

const PEER_ADDRESS: &str = "0.0.0.0:50000";

fn run_peer(remote: SocketAddr) -> io::Result<()> {
    let socket = UdpSocket::bind(PEER_ADDRESS)?;
    println!("peer iniciado em {}", socket.local_addr()?);
    println!("endpoint do outro peer: {remote}");

    let mut buffer = [0_u8; 128];

    for attempt in 1..=20 {
        let message = format!("hello {attempt}");
        println!("tentativa {attempt}: enviando para {remote}");
        socket.send_to(message.as_bytes(), remote)?;
        socket.set_read_timeout(Some(Duration::from_millis(500)))?;

        match socket.recv_from(&mut buffer) {
            Ok((size, source)) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("recebido de {source}: {message}");

                if message != "ack" {
                    socket.send_to(b"ack", source)?;
                }

                return Ok(());
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                println!("tentativa {attempt}: sem resposta")
            }
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "não foi possível estabelecer o caminho direto",
    ))
}

fn main() -> io::Result<()> {
    let mut arguments = env::args();
    let program = arguments.next().unwrap_or_else(|| "hole-punch".to_string());

    match (arguments.next(), arguments.next()) {
        (Some(remote), None) => {
            let remote = remote
                .parse()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
            run_peer(remote)
        }
        _ => {
            eprintln!("uso: {program} IP_PUBLICO:PORTA");
            Ok(())
        }
    }
}
