use tokio::net::UdpSocket;
use core::num;
//use std::arch::aarch64::__breakpoint;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use std::string;
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
 // to identify the ip address of the machine this code is running on
use local_ip_address::local_ip;

// Struct to represent server information
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ServerInfo {
    address: String,
    size: u8,
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
    let client_socket = UdpSocket::bind(&client_port).await?;
    let mut client_buffer = [0; 4096];
    println!("The clients' port is listening on: {}", client_port);
    println!("------------------------");
    //connect to server socket
    let server_port_num = ":10040";
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
    };
    serv_struct_vec.push(my_server_struct); 
    //add the other servers read from DoSS to the vector of structs
    for addr in &server_addr_v{
        let servers_struct = ServerInfo {
            address: addr.to_string(),
            size: 0,
        };
        serv_struct_vec.push(servers_struct); 
    }

    // Print server information
    println!("My server's port is listening on: {}", server_port);
    println!("------------------------");

    //Note: We would need to figure out a way to work around a server being down
    //We could do this by removing the down server from the vector and when it
    //comes back up it would be able to communicat with the other servers
    //so we will be able to add it back once a message is received from a server that is not in the vector


    //every server is going to receive the image from the client and then we will do leader election
    //to decide on which server is going to process the image.
    let mut image_num:u32 = 0;
    let mut num_requests = 0;
    loop {
        client_buffer = [0; 4096];
        // let mut received_data = Vec::new();
        let image_string = image_num.to_string();
        let image_name = "img_rcv".to_string() + &image_string + ".jpeg";
        let mut file = File::create(image_name)?;
        loop{
            //receive message from client
            let (len, client) = client_socket.recv_from(&mut client_buffer).await?;
            println!("Received {} bytes from {}", len, client);
            file.write_all(&client_buffer[..len])?;
            // println!("Received string: {}", client);
            // breah after the last packet
            if len != 4096 {
                break;
        }
        }
        image_num = image_num + 1;
        num_requests = num_requests + 1;
        // let message_client = num_requests;
        println!("------------------------");

        let my_serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server_port).unwrap();
        my_serv.size = num_requests as u8;
        
        let message_str = num_requests.to_string();
        // let m = "Hello, yasta!";
        let message_size_bytes = message_str.as_bytes();
        for addr in &server_addr_v{
            server_socket.send_to(message_size_bytes, &addr).await?;
            println!("Sent my buffer size of: {} to server {}", message_str, addr);
        }
        println!("------------------------");

        // add my own buffer size to the struct vector
        //value moved here in previous iteration of the loop

        for addr in &server_addr_v{
            //receive the buffer size from other servers using the same port
            let a = addr;
            println!("Waiting for a other server's message...");

            let (len, server) = server_socket.recv_from(&mut server_buffer).await?;
            // receive the buffer size from the server as
            let message_server = std::str::from_utf8(&server_buffer[..len])?;
            println!("Received the buffer size of: {} from server {}", message_server, server);
            //struct to add to the vector so that we can sort it
            //should be moved outside the loop
            //CHECK THIS

            println!("------------------------");
            let serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server.to_string()).unwrap();
            serv.size = message_server.parse().unwrap();

       }
    
    serv_struct_vec.sort_by(compare_servers);
    // serv_struct_vec.sort_by_key(|ser| ser.size);
    println!("The sorted vector before election is: ");
    for serv in &serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
    //    //check if the lowest size is my size and the tie case
    let mut i = 0;
    if serv_struct_vec[0].size as usize == num_requests as usize{
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

                let message3 = "Hello, I am the leader, aka, your mother!";

                let message_bytes3 = message3.as_bytes(); 
                //send the message to the client
                // client_socket.send_to(message_bytes3, &client_port).await?;
                // println!("Sent: {} to {}",  message3, client_port);
            }
            else {
            num_requests = num_requests - 1;
            }
        }
    else {
        num_requests = num_requests - 1;
        }
    
    println!("The sorted vector after election is: ");
    for serv in &serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
}

    Ok(())
}

