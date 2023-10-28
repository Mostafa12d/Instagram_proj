// This is the client
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //Specify address form file
    // let filename = "DoSC.txt";
    // let file = File::open(filename).expect("no such file");
    // let buf = BufReader::new(file);
    // let local_addr_v: Vec<String> = buf.lines()
    // .map(|l| l.expect("Could not parse line"))
    // .collect();
    // for addr in &local_addr_v{
    //     println!("{}", addr);
    // }

    //Specify address from command line
    // let args: Vec<String> = std::env::args().collect();
    // let local_addr = args.get(1).expect("Argument 1 is listening address. Eg: 0.0.0.0:10001");

    // //Specify address from code
    let local_addr = "172.29.255.134:10011"; 

    //original
    let remote_addr1 = "172.29.255.134:8092"; // IP address and port of the Server 1
    let remote_addr2 = "172.29.255.134:8093"; // IP address and port of the Server 2
    let remote_addr3 = "172.29.255.134:8094"; // IP address and port of the Server 3

    
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let message = "Hello, Server!";
    let message_bytes = message.as_bytes();  

    // socket.connect(local_addr).await?; // Connect to the server
    // socket.send(message_bytes).await?; // Send the message                

    socket.send_to(message_bytes, local_addr).await?; // Send the message

    // socket.send_to(message_bytes, remote_addr3).await?;
    // socket.send_to(message_bytes, remote_addr2).await?;
    // socket.send_to(message_bytes, remote_addr1).await?;

    println!("Sent: {} to {}", message, local_addr);
    
    // println!("Sent: {} to {}", message, remote_addr1);
    // println!("Sent: {} to {}", message, remote_addr2);
    // println!("Sent: {} to {}", message, remote_addr3);

    //Receive Reply from server
    let mut buffer = [0; 1024]; // Buffer to receive the message

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);
    }

    Ok(())
}