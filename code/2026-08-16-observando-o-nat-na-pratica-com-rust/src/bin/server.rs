use std::io;
use std::net::UdpSocket;

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:40000")?;
    let local = socket.local_addr()?;
    println!("servidor iniciado em {local}");

    let mut buffer = [0_u8; 1024];

    loop {
        let (size, source) = socket.recv_from(&mut buffer)?;
        let message = String::from_utf8_lossy(&buffer[..size]);

        println!("recebido de {source}: {message}");
        socket.send_to(source.to_string().as_bytes(), source)?;
    }
}