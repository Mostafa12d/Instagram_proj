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
    let remote_addr1 = "192.168.100.74:10014"; // IP address and port of the Server 1
    let remote_addr2 = "172.20.10.4:10019"; // IP address and port of the Server 2
    let remote_addr3 = "172.20.10.4:10020"; // IP address and port of the Server 3

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let message1 = "Hello, Server18!";
    let message2 = "Hello, Server19!";
    let message3 = "Hello, Server20!";

    // let message_bytes1 = message1.as_bytes();  
    let message_bytes2 = message2.as_bytes();  
    let message_bytes3 = message3.as_bytes();  

    // socket.send_to(message_bytes1, remote_addr1).await?; // Send the message
    socket.send_to(message_bytes2, remote_addr2).await?;
    socket.send_to(message_bytes3, remote_addr3).await?;
    
    println!("Sent: {} to {}", message1, remote_addr1);
    println!("Sent: {} to {}", message2, remote_addr2);
    println!("Sent: {} to {}", message3, remote_addr3);


//function to send an image
    let image_path = "./src/bambo.jpeg";
    let mut img = File::open(image_path)?;
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer)?;
    println!("Image Buffer content: {:?}", buffer);
    for chunk in buffer.chunks(4096) {
        //Receive Reply from server
        socket.send_to(chunk, remote_addr1).await?;
        sleep(Duration::from_secs(1)).await;
    }


    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);
    }

    Ok(())
}