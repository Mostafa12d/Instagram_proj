use tokio::net::UdpSocket;
//use std::arch::aarch64::__breakpoint;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use std::string;
use rand::seq::SliceRandom;
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

// Implement the From_String trait for the ServerInfo struct
// impl ServerInfo {
//     fn from_string(s: &str) -> ServerInfo {
//         let parts: Vec<&str> = s.split_whitespace().collect();
//         let ip = parts[1].to_string();
//         let port = parts[3].parse().unwrap();
//         let size = parts[5].parse().unwrap();

//         ServerInfo {address, size }
//     }
// }

// Append server information to a txt file
fn get_server_info(filename: &str) -> Vec<String> {

    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    let local_addr_v: Vec<String> = buf.lines()
    .map(|l| l.expect("Could not parse line"))
    .collect();

    
    // for addr in &local_addr_v{
    //     println!("{}", addr);
    // }
    //return server addresses
    local_addr_v
}

// Start the server
async fn start_server(local_addr: &str) -> Result<(), Box<dyn Error>> {
    //connect to client socket
    let client_port = local_addr.to_string()+":10018";
    let client_socket = UdpSocket::bind(&client_port).await?;
    let mut client_buffer = [0; 1024];
    println!("The clients' port is listening on: {}", client_port);
    println!("------------------------");
    //connect to server socket
    let server_port_num = ":10010";
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
    //create a vector that holds the messages
    //ERROR shared vector in an async function(explore threads later)
    let mut message_buffer = Vec::new();


    //Note: We would need to figure out a way to work around a server being down
    //We could do this by removing the down server from the vector and when it
    //comes back up it would be able to communicat with the other servers
    //so we will be able to add it back once a message is received from a server that is not in the vector

    loop {
        //receive message from client
        let (len, client) = client_socket.recv_from(&mut client_buffer).await?;
        let message_client = std::str::from_utf8(&client_buffer[..len])?;
        println!("Received buffer size of: {} from {}", message_client, client);
        println!("------------------------");

        let temp_message_client = message_client.to_string();
        //add message to buffer
        //ERROR shared vector in an async function(explore threads later)
        message_buffer.push(temp_message_client);

        //send the buffer size to other servers using the same port
        let message_size = message_buffer.len();

        let my_serv = serv_struct_vec.iter_mut().find(|serv| serv.address == server_port).unwrap();
        my_serv.size = message_size as u8;
        
        let message_str = message_size.to_string();
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

       //to update a certain entry in the struct
       //if let Some(index) = people.iter().position(|person| person.name == name_to_update) {
        // Update the age field for the matching person
        // people[index].age = new_age;
    // }
    //print the addresses of the
    
    serv_struct_vec.sort();
    
    for serv in &serv_struct_vec{
        println!("{} {}", serv.address, serv.size);
    }
       //check if the lowest size is my size and the tie case
       let mut i = 0;
       if serv_struct_vec[0].size as usize == message_size{
            while serv_struct_vec[i].size == serv_struct_vec[0].size{
                if i < serv_struct_vec.len()-1{
                    i = i+1;
                }
        }
            let server_seed = 42; // Replace this with the synchronized seed for each server
            let random_number = generate_random_number(server_seed, i);
            let chosen_server = serv_struct_vec[random_number as usize].address.clone();
            if chosen_server == server_port{
                //execute
                println!("I am {} the chosen server ", chosen_server);

                let message3 = "Hello, I am leader how is and your mother!";

                let message_bytes3 = message3.as_bytes(); 
                //send the message to the client
                client_socket.send_to(message_bytes3, &client).await?;
                println!("Sent: {} to {}",  message3, client);
            }
            else{
                //send the message to the chosen server
                println!("I am {} not the chosen server ", chosen_server);
                server_socket.send_to(message_size_bytes, &chosen_server).await?;
                println!("Sent: {} to {}", message_str, chosen_server);
            }
       }


    //    if message_size > serv_struct_vec[0]{
    //        //execute
    //    }
    //    else if message_size == serv_struct_vec[0]{
           
    //    }

        // //let least_busy = std::cmp::min(server_addr_v[0], std::cmp::min(b, c));
        // if message_size < serv_struct_vec[0] && message_size < serv_struct_vec[1]{
        //     //execute
        // }
        // else if message_size == serv_struct_vec[0] && message_size == serv_struct_vec[1]{
            
        // }
        // else if message_size == serv_struct_vec[0]{

        // }
        // else if message_size == serv_struct_vec[1]{
            
        // }


        // else {
            
        // }


        // let buf1_size: u8 = message_server.parse().unwrap();

        
        // add messages to a message buffer
        // compare buffer sizes once it happens
        //if your buffer is more than any of the other buffer, delete the last message
        //if not continue with processing the buffer
        //if(buf1 < buf2< buf3)

        
        //// Draft Leader Election
        //check which server is free using some ifs
        //send token = ok to the server that has the least priority

    }

    Ok(())
}

// !!!!!!!!! DOES NOT WORK PROPERLY NEEDS FIXING!!!!!!!!!
// fn set_server_status(server_info: &ServerInfo, new_status: u8) -> Result<(), Box<dyn Error>> {
//     // let file_contents = {
//     //     let file_contents = std::fs::read_to_string("DoSS.txt")?;
//     //     file_contents
//     // };

//     // let new_server_info = format!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, new_status);
//     // let modified_contents = file_contents.replace(&new_server_info, "");

//     // // let mut file = File::create("DoSS.txt")?;
//     // // file.write_all(modified_contents.as_bytes())?;

//     // Ok(())
    
//     let filename = "DoSS.txt";
//     let _file = File::open(filename).expect("no such file");
//     // let buf = BufReader::new(file);
//     // let buf = BufReader::new(file);
//     // let file_contents: Vec<String> = buf.lines()
//     // .map(|l| l.expect("Could not parse line"))
//     // .collect();

//     let file_contents = std::fs::read_to_string(filename)?;



//     let new_server_info = format!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, new_status);
    
//     let modified_contents = file_contents.replace(
//         &format!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, server_info.size),
//         &new_server_info,
//     );

//     let mut file = File::create("DoSS.txt")?;
//     file.write(modified_contents.as_bytes()).expect("failed writing");

//     Ok(())
// }


// fn find_active_server() -> Result<Option<ServerInfo>, Box<dyn Error>> {
//     let file = File::open("/home/tamer/DS/Instagram_proj/Proj/Server/src/DoSS.txt")?;
//     let reader = BufReader::new(file);

//     let active_servers: Vec<ServerInfo> = reader
//         .lines()
//         .map(|line| line.expect("Could not parse line"))
//         .filter_map(|line| {
//             let server_info = ServerInfo::from_string(&line);
//             if server_info.size == 1 {
//                 Some(server_info)
//             } else {
//                 None
//             }
//         })
//         .collect();

//     if active_servers.is_empty() {
//         Ok(None)
//     } else {
//         let mut rng = rand::thread_rng();
//         if let Some(chosen_server) = active_servers.choose(&mut rng) {
//             Ok(Some(chosen_server.clone()))
//         } else {
//             Err("Failed to select a random server".into())
//         }
//     }
// }

