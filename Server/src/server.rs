use tokio::net::UdpSocket;
use tokio::time::{self, sleep, Duration};
use core::num;
use std::io::Read;
use steganography::util::save_image_buffer;
//use std::arch::aarch64::__breakpoint;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use std::{string, result};
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
// use tokio::time::{self, Duration};
use std::thread;
use image::DynamicImage;
use steganography::decoder::*;
use steganography::encoder::*;
use sysinfo::{CpuExt, System, SystemExt};
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
// Start the server
async fn start_server(local_addr: &str) -> Result<(), Box<dyn Error>> {
    //connect to client socket
    let client_port = local_addr.to_string()+":10014";
    let client_port_send = local_addr.to_string()+":10000";
    let client_socket = UdpSocket::bind(&client_port).await?;
    let mut client_socket_send = UdpSocket::bind(&client_port_send).await?;
    let mut client_buffer = [0; 4096];
    println!("The clients' port is listening on: {}", client_port);
    println!("------------------------");
    //connect to server socket
    let server_port_num = ":10045";
    let server_port = local_addr.to_string()+server_port_num;
    let server_socket = UdpSocket::bind(&server_port).await?;
    let mut server_buffer = [0; 1024];

    //get the available servers
    let mut serv_struct_vec = Vec::new();
    let mut server_addr_v = Vec::new();
    server_addr_v = get_server_info("./src/DoSS.txt");

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
    println!("My server's port is listening on: {}", server_port);
    println!("------------------------");
    // Get the CPU information
    let mut sys = System::new_all();
    sys.refresh_all();
    let process_count = sys.processes().len();
    println!("=> System:");
    println!("Number of running processes: {}", process_count);
    println!("------------------------");   
    //every server is going to receive the image from the client and then we will do leader election
    //to decide on which server is going to process the image.
    let mut image_num:u32 = 0;
    let mut num_requests = 0;
    let mut client_address;

    //client send and receive thread
    loop {
        client_buffer = [0; 4096];
        let mut packet_buffer = [0; 4104];
        let mut processing_buffer = [0; 4104];
        //let mut client_message = Vec::new();
        // let mut received_data = Vec::new();
        let mut last_received_sequence_number: u64 = 0;
        let image_string = image_num.to_string();
        let image_name = "imgs/img_rcv".to_string() + &image_string + ".jpeg";
        let image_name2 = image_name.clone();
        let mut file = File::create(image_name)?;
        let mut i = 0;
        loop{
            //receive message from client
            i+=1;
            println!("i = {}", i);
            let (len, client) = client_socket.recv_from(&mut packet_buffer).await?;
            let length = len;
            processing_buffer = packet_buffer;
            let received_sequence_number = u64::from_be_bytes(processing_buffer[0..8].try_into().unwrap());
            println!("Packet sequence number: {}", received_sequence_number);
            println!("Last received sequence number: {}", last_received_sequence_number);

            let mut missing_packets_string = String::new();
            if received_sequence_number != last_received_sequence_number + 1 {
                println!("Received out of order packet");
                for seq_num in last_received_sequence_number + 1..received_sequence_number {
                    //push number as 3 digit string
                    missing_packets_string.push_str(&format!("{:03}", seq_num));
                }
             }

            let data_length = len - 8; // Adjust for the sequence number
            // Make sure we don't exceed the bounds of either buffer
            let copy_length = std::cmp::min(data_length, client_buffer.len());
             // Now safely copy the data
            client_buffer[..copy_length].copy_from_slice(&processing_buffer[8..8 + copy_length]);         
            println!("client buffer size {}", client_buffer.len());
            println!("Received {} bytes from {}", client_buffer.len(), client);
            client_address = client;
            file.write_all(&client_buffer[..len-8])?;
            //client_message.push(client_buffer[..len-8].to_vec());
            // println!("Received string: {}", client);
            // break after the last packet
            // print the packet_buffer length
            println!("Packet buffer length: {}", length);
            if length < 4104 {
                break;
            }
            last_received_sequence_number = received_sequence_number;
        }
        image_num = image_num + 1;
        num_requests = num_requests + 1;
        // let message_client = num_requests;
        println!("------------------------");

        //fault tolerance thread
        let my_serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server_port).unwrap();
        my_serv.size = num_requests + process_count as u32;
        let mut server_load =  my_serv.size.clone();

        
        let message_str = server_load.to_string();
        // let m = "Hello, yasta!";
        let message_size_bytes = message_str.as_bytes();
        for addr in &server_addr_v{
            server_socket.send_to(message_size_bytes, &addr).await?;
            println!("Sent my buffer size of: {} to server {}", message_str, addr);
        }
        println!("------------------------");
        //vector to add all the servers that sent messages
        let mut received_servers = Vec::new();
        for saddr in &server_addr_v{
            //if the server is not responding for 0.5 seconds, then we will assume that it is down
            match time::timeout(Duration::from_millis(200), server_socket.recv_from(&mut server_buffer)).await{
                Ok(Ok((len, server))) => {
                    let message_server = std::str::from_utf8(&server_buffer[..len])?;
                    println!("Received the buffer size of: {} from server {}", message_server, server);
                    let serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server.to_string()).unwrap();
                    serv.size = message_server.parse().unwrap();
                    //add received server to the vector
                    received_servers.push(server.to_string());
                }
                Ok(Err(_)) | Err(_) => {
                    eprintln!("Timeout reached while waiting for data");
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
    for serv in &serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
    //check if the lowest size is my size and the tie case
    let mut i = 0;
    if serv_struct_vec[0].size as usize == server_load as usize{
        for serv in &serv_struct_vec{
             if serv.size == serv_struct_vec[0].size{
                 i = i+1;
             }
            }
            println!("{}", i);
            let server_seed = 42; // Replace this with the synchronized seed for each server
            let random_number = generate_random_number(server_seed, i);
            println!("random number {}", random_number);

            let chosen_server = serv_struct_vec[random_number as usize].address.clone();
            if chosen_server == server_port{
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
            let mut secret_image = File::open(image_name2).unwrap(); 
            let cover = image::open("./src/loading.png").unwrap();
            let mut secret_image_vec = Vec::new();  
            // let mut secret = File::open("./src/yaboy.jpg").unwrap();
            secret_image.read_to_end(&mut secret_image_vec).unwrap();
            
            let encoders = Encoder::new(&secret_image_vec, cover);

            let encoded_image = encoders.encode_alpha();
            save_image_buffer(encoded_image.clone(), "./src/encoded.png".to_string());

            let mut encoded = File::open("./src/encoded.png").unwrap(); 
            let mut encoded_vec = Vec::new();  
            // let mut secret = File::open("./src/yaboy.jpg").unwrap();
            encoded.read_to_end(&mut encoded_vec).unwrap();
                    for chunk in encoded_vec.chunks(4096){
                    //send packets to server
                    println!("Sending chunk of: {} to {}", chunk.len(), client_address);
                    client_socket_send.send_to(chunk, &client_address).await?;
                    if chunk.len() != 4096 {
                       break;
                    }
                    sleep(Duration::from_millis(5)).await;
                //     // let delay = time::Duration::from_millis(1000);
                //     // time::sleep(delay).await;
                // }
                //}

            }
        }
            else {
            num_requests = num_requests - 1;
            //delete the file stored
            // std::fs::remove_file(image_name2)?;
            }
        }
    else {
        num_requests = num_requests - 1;
        // std::fs::remove_file(image_name2)?;
        }
    //encryption and resending thread

    println!("The sorted vector after election is: ");
    for serv in &serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
}

    Ok(())
}

