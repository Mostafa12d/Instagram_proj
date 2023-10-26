// This is the client
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //Specify address form file
    let filename = "DoSC.txt";
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

    //original
    let remote_addr = "127.0.0.1:8080"; // IP address and port of the Server
    
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let message = "Hello, World!";
    let message_bytes = message.as_bytes();

    socket.send_to(message_bytes, remote_addr).await?;

    println!("Sent: {}", message);

    Ok(())
}