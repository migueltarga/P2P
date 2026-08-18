use std::io::{self, Write};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use stunclient::StunClient;

const LOCAL_ADDRESS: &str = "0.0.0.0:50000";
const STUN_ADDRESS: &str = "stun.cloudflare.com:3478";

fn resolve_stun_server() -> io::Result<SocketAddr> {
    STUN_ADDRESS
        .to_socket_addrs()?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "STUN sem IPv4"))
}

fn read_remote_endpoint() -> io::Result<SocketAddr> {
    print!("endpoint externo do outro peer: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input
        .trim()
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

fn punch(socket: &UdpSocket, remote: SocketAddr) -> io::Result<()> {
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
    let socket = UdpSocket::bind(LOCAL_ADDRESS)?;
    let stun_server = resolve_stun_server()?;
    let external = StunClient::new(stun_server)
        .query_external_address(&socket)
        .map_err(|error| io::Error::other(error.to_string()))?;

    println!("socket local: {}", socket.local_addr()?);
    println!("endpoint externo: {external}");
    println!("envie o endpoint externo ao outro peer");

    let remote = read_remote_endpoint()?;
    punch(&socket, remote)
}
