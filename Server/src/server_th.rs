use tokio::net::UdpSocket;
use core::num;
//use std::arch::aarch64::__breakpoint;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use std::{string, result};
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use tokio::time::{self, Duration};
use std::thread;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
 // to identify the ip address of the machine this code is running on
use local_ip_address::local_ip;

// Struct to represent server information
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ServerInfo {
    address: String,
    size: u32,
    status: u8,
}


fn generate_random_number(seed: u64, i: usize) -> u64 {
    let mut rng = SmallRng::seed_from_u64(seed);
    rng.gen::<u64>()%i as u64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // // Get Port from Command Line
    // let args: Vec<String> = std::env::args().collect();
    // let port_num = args.get(1).expect("Argument 1 is listening port. Eg: 8080");
    // println!("{}", port_num);
    //Function finds the ip of the running server
    let local_ip = local_ip().unwrap(); // Get the dynamically assigned IP address
    // Create a server
    let local_addr = local_ip.to_string();//+":"+port_num;
    println!("{}", local_addr);
    // Start the server
    start_server(&local_addr).await?;

    Ok(())
}

// Append server information to a txt file
fn get_server_info(filename: &str) -> Vec<String> {

    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    let local_addr_v: Vec<String> = buf.lines()
    .map(|l| l.expect("Could not parse line"))
    .collect();

    //return server addresses
    local_addr_v
}

fn compare_servers(a: &ServerInfo, b: &ServerInfo) -> Ordering {
    let a_ip = a.address.split(':').next().unwrap();
    let a_port = a.address.split(':').last().unwrap().parse::<u16>().unwrap();
    let b_ip = b.address.split(':').next().unwrap();
    let b_port = b.address.split(':').last().unwrap().parse::<u16>().unwrap();

    a.size.cmp(&b.size)
        .then(a_ip.cmp(&b_ip))
        .then(a_port.cmp(&b_port))
}
async fn receive_imgs(client_socket: &UdpSocket, num_requests: &mut u32, file: &mut File) {
        let mut image_num = 0;
        let mut client_buffer = [0; 4096];
        let mut packet_buffer = [0; 4104];
        let mut client_message = Vec::new();
        // let mut received_data = Vec::new();
        let mut last_received_sequence_number: u64 = 0;
        // let image_string = image_num.to_string();
        // let image_name = "imgs/img_rcv".to_string() + &image_string + ".jpeg";
        // let image_name2 = image_name.clone();
        let mut i = 0;
        loop{
            //receive message from client
            i+=1;
            println!("i = {}", i);
            match client_socket.recv_from(&mut packet_buffer).await {
                Ok((len, client)) => {
                let received_sequence_number = u64::from_be_bytes(packet_buffer[0..8].try_into().unwrap());
                println!("Packet sequence number: {}", received_sequence_number);
                println!("Last received sequence number: {}", last_received_sequence_number);
    
                let mut missing_packets_string = String::new();
                if received_sequence_number != last_received_sequence_number + 1 {
                    println!("Received out of order packet");
                    for seq_num in last_received_sequence_number + 1..received_sequence_number {
                        //push number as 3 digit string
                        missing_packets_string.push_str(&format!("{:03}", seq_num));
                    }
                    // println!("Missing packets: {:?}", missing_packets_string); 
                    
                //     let byte_vec: Vec<u8> = missing_packets.iter()
                //    .flat_map(|&num| num.to_be_bytes())
                //     .collect();
    
    
               // let missing_packets_string = String::from_utf8(missing_packets).expect("Invalid UTF-8");
            //    let message_bytes = missing_packets_string.as_bytes();
            //    //send NACK message to client
            //    //println!("Missing packets: {:?}", missing_packets_string); 
            //    client_socket.send_to(&message_bytes, &client).await?;
    
                }
                // else{
                //     missing_packets_string.push_str("000");
                //     let message_bytes = missing_packets_string.as_bytes();
                //     client_socket.send_to(&message_bytes, &client).await?;
    
                // }
                client_buffer.copy_from_slice(&packet_buffer[8..len]);
                println!("client buffer size {}", client_buffer.len());
                println!("Received {} bytes from {}", client_buffer.len(), client);
                // client_address = client;
                client_message.push(client_buffer[..len-8].to_vec());
                file.write_all(&client_buffer[..len-8]);
                // println!("Received string: {}", client);
                // breah after the last packet
                if i == 76 {
                    break;
                }
                last_received_sequence_number = received_sequence_number;
            }
            Err(e) => {
                eprintln!("Error receiving from client: {}", e);
            }
        }
        }
        image_num = image_num + 1;
        *num_requests +=1;
        // let message_client = num_requests;
        println!("------------------------");
} 
async fn leader_election(server_socket: &UdpSocket, server_port: &String, server_addr_v: &Vec<String>, serv_struct_vec: &mut Vec<ServerInfo>, num_requests: &mut u32){
    let mut server_buffer = [0; 1024];
    let my_serv = serv_struct_vec.iter_mut().find(|serv| serv.address == *server_port).unwrap();
        my_serv.size = *num_requests as u32;
        
        let message_str = num_requests.to_string();
        // let m = "Hello, yasta!";
        let message_size_bytes = message_str.as_bytes();
        for addr in server_addr_v{
            server_socket.send_to(message_size_bytes, &addr).await;
            println!("Sent my buffer size of: {} to server {}", message_str, addr);
        }
        println!("------------------------");
        //vector to add all the servers that sent messages
        let mut received_servers = Vec::new();
        for saddr in server_addr_v{
            //if the server is not responding for 0.5 seconds, then we will assume that it is down
            match time::timeout(Duration::from_millis(10), server_socket.recv_from(&mut server_buffer)).await{
                Ok(result) => match result {
                    Ok((len, server)) => {
                    let message_server = std::str::from_utf8(&server_buffer[..len]).unwrap();
                    println!("Received the buffer size of: {} from server {}", message_server, server);
                    let serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server.to_string()).unwrap();
                    serv.size = message_server.parse().unwrap();
                    //add received server to the vector
                    received_servers.push(server.to_string());
                    },
                    Err(e) => {
                        eprintln!("Error receiving from server: {}", e);
                    }
                },
                Err(_)=> {
                    panic!("Timeout reached while waiting for data");
                }
        }
        }
        //check if there are servers that did not send a message
        let diff: Vec<_> = server_addr_v.iter().filter(|&item| !received_servers.contains(item)).cloned().collect();
        for addr in diff{
            let temp_serv = serv_struct_vec.iter_mut().find(|serv| serv.address ==  addr).unwrap();
            //change their size to 255 which means unavailable
            temp_serv.size = 9999;
        }

    //election thread
    serv_struct_vec.sort_by(compare_servers);
    println!("The sorted vector before election is: ");
    let first_size = serv_struct_vec.first().map(|s| s.size as usize).unwrap_or(0);
    for serv in &mut *serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
    //check if the lowest size is my size and the tie case
    let mut i = 0;
    if first_size as usize == *num_requests as usize{
        for serv in &mut *serv_struct_vec{
             if serv.size == first_size as u32{
                 i = i+1;
             }
            }
            println!("{}", i);
            let server_seed = 42; // Replace this with the synchronized seed for each server
            let random_number = generate_random_number(server_seed, i);
            println!("random number {}", random_number);

            let chosen_server = serv_struct_vec[random_number as usize].address.clone();
            if chosen_server == *server_port{
                //execute
                println!("I am {} the chosen server ", chosen_server);

                // let message3 = "Hello, I am the leader, aka, your mother!";
                // let message_bytes3 = message3.as_bytes(); 
                //send the message to the client+
                //function to send an image
                // let image_path = "./src/bambo.jpeg";
                // let mut img = File::open(image_path)?;
                // let mut buffer = Vec::new();
                // println!("Image Buffer content: {:?}", buffer);
                // iterate 400 times in a for loop
                // img.read_to_end(&mut client_buffer)?;
                //for i in 0..100 {    
                    // for chunk in &client_message {
                    // //send packets to server
                    // println!("Sending chunk of: {} to {}", chunk.len(), client_address);
                    // client_socket_send.send_to(chunk, &client_address).await?;
                    // if chunk.len() != 4096 {
                    //    break;
                    // }
                //     // let delay = time::Duration::from_millis(1000);
                //     // time::sleep(delay).await;
                // }
                //}

            }
            else {
            *num_requests -= 1;
            //delete the file stored
            // std::fs::remove_file(image_name2)?;
            }
        }
    else {
       *num_requests -= 1;
        // std::fs::remove_file(image_name2)?;
        }

}
// Start the server
async fn start_server(local_addr: &str) -> Result<(), Box<dyn Error>> {
    //connect to client socket
    let client_port = local_addr.to_string()+":10017";
    let client_port_send = local_addr.to_string()+":10000";
    let client_socket = UdpSocket::bind(&client_port).await?;
    let mut client_socket_send = UdpSocket::bind(&client_port_send).await?;
    // let mut client_buffer = [0; 4096];
    println!("The clients' port is listening on: {}", client_port);
    println!("------------------------");
    //connect to server socket
    let server_port_num = ":10045";
    let server_port = local_addr.to_string()+server_port_num;
    let server_socket = UdpSocket::bind(&server_port).await?;

    //get the available servers
    let mut serv_struct_vec = Vec::new();
    let mut server_addr_v = Vec::new();
    server_addr_v = get_server_info("./src/DoSS.txt");
    // let server_addr_v_th = server_addr_v.clone();
    //add my server to the vector of structs
    let my_server_struct = ServerInfo {
        address: server_port.clone(),
        size: 0,
        status: 1,
    };
    serv_struct_vec.push(my_server_struct); 
    //add the other servers read from DoSS to the vector of structs
    for addr in &server_addr_v{
        let servers_struct = ServerInfo {
            address: addr.to_string(),
            size: 0,
            status: 1,
        };
        serv_struct_vec.push(servers_struct); 
    }
    // Print server information
    println!("My server's port is listening on: {}", server_port.clone());
    println!("------------------------");

    //every server is going to receive the image from the client and then we will do leader election
    //to decide on which server is going to process the image.
    let mut image_num:u32 = 0;
    let mut num_requests: u32 = 0;
    // let mut client_address;

    //client send and receive thread
    let client_socket = Arc::new(client_socket);
    let server_socket = Arc::new(server_socket);
    let server_port_arc = Arc::new(server_port);
    let serv_struct_vec = Arc::new(Mutex::new(serv_struct_vec));

    loop {
        let client_socket = Arc::clone(&client_socket);
        let image_string = image_num.to_string();
        let image_name = "imgs/img_rcv".to_string() + &image_string + ".jpeg";
        let image_name2 = image_name.clone();
        let mut file = File::create(image_name)?;
        let client_thread = task::spawn(async move{
            receive_imgs(&client_socket, &mut num_requests, &mut file).await;
        });
        
        //fault tolerance thread
        let server_addr_v_th = server_addr_v.clone();
        let serv_struct_vec_clone = Arc::clone(&serv_struct_vec);
        let server_port_clone = Arc::clone(&server_port_arc);
        let server_socket = Arc::clone(&server_socket);
        let server_port_clone = Arc::try_unwrap(server_port_clone).unwrap_or_else(|arc| (*arc).clone());
        let election_thread = task::spawn(async move{
            let mut serv_struct_vec = serv_struct_vec_clone.lock().await;
        leader_election(&server_socket, &server_port_clone, &server_addr_v_th, &mut *serv_struct_vec, &mut num_requests).await
    });
    //encryption and resending thread

    // println!("The sorted vector after election is: ");
    // for serv in &serv_struct_vec{
    //     println!("{} {}", serv.address, serv.size);
    // }
}

    Ok(())
}

