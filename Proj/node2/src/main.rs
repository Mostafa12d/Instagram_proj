use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{Read, Result};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote_addr = "127.0.0.1:8080"; // IP address and port of Node 2
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let message = "Hello, World!";
    
    //Read the picture and change it to a vector of bytes
    let mut file = File::open("tremer.png")?;
    let mut picture_data = Vec::new();
    file.read_to_end(&mut picture_data)?;
    let message_bytes = message.as_bytes();

    socket.send_to(message_bytes, remote_addr).await?;

    println!("Sent: {}", message);

    Ok(())
}



