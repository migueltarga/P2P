use std::env;
use std::io;
use std::net::UdpSocket;
use std::time::Duration;

fn main() -> io::Result<()> {
    let server = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:40000".to_string());

    let socket = UdpSocket::bind("0.0.0.0:50000")?;
    socket.connect(&server)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;

    println!("endpoint local: {}", socket.local_addr()?);
    println!("destino: {}", socket.peer_addr()?);

    socket.send(b"ola")?;

    let mut buffer = [0_u8; 1024];
    let size = socket.recv(&mut buffer)?;
    let observed = String::from_utf8_lossy(&buffer[..size]);

    println!("origem identificada pelo servidor: {observed}");
    Ok(())
}