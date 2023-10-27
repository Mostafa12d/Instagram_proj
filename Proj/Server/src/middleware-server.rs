// This is the server
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::net::{SocketAddr, Ipv4Addr}; // to identify the ip address of the machine this code is running on
use local_ip_address::local_ip;
use core::cmp::Ordering;
use rand::Rng;



#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // //Specify address form file
    let filename = "DoSS.txt";
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    let servers: Vec<String> = buf.lines()
    .map(|l| l.expect("Could not parse line"))
    .collect();
    println!("Servers currently up:");
    for addr in &servers{
        println!("{}", addr);
    }
    //let local_addr = &local_addr_v[0];

    // Specify address from command line
    let args: Vec<String> = std::env::args().collect();
    //let local_addr = args.get(1).expect("Argument 1 is listening address. Eg: 0.0.0.0:10001");
    let local_addr = local_ip().unwrap(); // Get the dynamically assigned IP address


    // //Specify address from code
    let local_add = "0.0.0.0:8093"; // IP address and port you want the server (this process) to listen on
    
    //bind sockets

    

    let socket = UdpSocket::bind(local_add).await?;
    let mut buffer = [0; 1024]; // Buffer to receive the message

    println!("This server is listening on: {}", local_addr);


    // Determine if this server is the leader based on the highest IP address
    let is_leader = is_leader(&servers);



    if is_leader {
        println!("I am the leader.");
    } else {
        println!("I am not the leader.");
    }


    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);

        // Send a response back to the client
        let response = if is_leader {
            "Hello, client! I am the leader."
        } else {
            "Hello, client! I am not the leader."
        };

        let sent_len = socket.send_to(response.as_bytes(), &src).await?;
        println!("Sent: {} bytes to {}", sent_len, src);
    }
}

fn is_leader(servers: &[String]) -> bool {
    // Randomly choose a leader
    let mut rng = rand::thread_rng();
    let leader_index = rng.gen_range(0..servers.len());
    leader_index == 0 // For example, the first server in the list is the leader
}