// This is the client
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::io::Cursor;
use std::io::Read;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //original
    let remote_addr1 = "192.168.100.74:10014"; // IP address and port of the Server 0
    let remote_addr2 = "192.168.100.74:10019"; // IP address and port of the Server 1
    let remote_addr3 = "192.168.100.74:10020"; // IP address and port of the Server 2

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
   let mut message_buffer = [0; 5000];

//function to send an image
    let image_path = "./src/bambo.jpeg";
    let mut img = File::open(image_path)?;
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer)?;
    // println!("Image Buffer content: {:?}", buffer);
    for chunk in buffer.chunks(4096) {
        //send packets to server
        println!("Sending chunk of: {}", chunk.len());
        socket.send_to(chunk, remote_addr1).await?;
        socket.send_to(chunk, remote_addr2).await?;
        socket.send_to(chunk, remote_addr3).await?;
    }


    loop {
        println!("Waiting for a message...");
        let (len, src) = socket.recv_from(&mut message_buffer).await?;
        let message = std::str::from_utf8(&message_buffer[..len])?;
        println!("Received: {} from {}", message, src);

    }

    Ok(())
}