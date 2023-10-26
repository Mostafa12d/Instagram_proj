// This is the server
use tokio::net::UdpSocket;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, BufRead};

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //Specify address form file
    let filename = "/Users/mostafalotfy/Documents/University/Fall 2023/Distributed Systems/Instagram_proj/Proj/DoS.txt";
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    let local_addr_v: Vec<String> = buf.lines()
    .map(|l| l.expect("Could not parse line"))
    .collect();
    for addr in &local_addr_v{
        println!("{}", addr);
    }

    //Specify address from command line
    // let args: Vec<String> = std::env::args().collect();
    // let local_addr = args.get(1).expect("Argument 1 is listening address. Eg: 0.0.0.0:10001");

    // //Specify address from code
    // let local_addr = "0.0.0.0:8086"; // IP address and port you want the server (this process) to listen on
    
    //bind sockets
    let local_addr = &local_addr_v[0];
    let socket = UdpSocket::bind(local_addr).await?;
    let mut buffer = [0; 1024]; // Buffer to receive the message

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);

        // Send a response back to the client
        let response = "Hello, client!"; // Change this to your response message
        let sent_len = socket.send_to(response.as_bytes(), &src).await?;
        println!("Sent: {} bytes to {}", sent_len, src);
    }
}
