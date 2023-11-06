use tokio::net::UdpSocket;
use std::arch::aarch64::__breakpoint;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use std::string;
use rand::seq::SliceRandom;
 // to identify the ip address of the machine this code is running on
use local_ip_address::local_ip;

// Struct to represent server information
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ServerInfo {
    address: String,
    size: u8,
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
    //connect to server socket
    let server_port_num = ":10010";
    let server_port = local_addr.to_string()+server_port_num;
    let server_socket = UdpSocket::bind(&server_port).await?;
    let mut server_buffer = [0; 1024];

    // Print server information
    println!("The servers' port is listening on: {}", server_port);
    //create a vector that holds the messages
    //ERROR shared vector in an async function(explore threads later)
    let mut message_buffer = Vec::new();

    //get the available servers
    let mut server_buf_size = Vec::new();
    let mut server_addr_v = Vec::new();
    server_addr_v = get_server_info("/Users/mostafalotfy/Documents/University/Fall 2023/Distributed Systems/Instagram_proj/Proj/Server/src/DoSS.txt");
    for addr in &server_addr_v{
        println!("{}", addr);
    }
    //Note: We would need to figure out a way to work around a server being down
    //We could do this by removing the down server from the vector and when it
    //comes back up it would be able to communicat with the other servers
    //so we will be able to add it back once a message is received from a server that is not in the vector

    loop {
        //receive message from client
        let (len, client) = client_socket.recv_from(&mut client_buffer).await?;
        let message_client = std::str::from_utf8(&client_buffer[..len])?;
        println!("Received: {} from {}", message_client, client);
        let temp_message_client = message_client.to_string();
        //add message to buffer
        //ERROR shared vector in an async function(explore threads later)
        message_buffer.push(temp_message_client);

        //send the buffer size to other servers using the same port
        
        //ERROR shared vector in an async function(explore threads later)
        let message_size = message_buffer.len();
        let message_str = message_size.to_string();
        // let m = "Hello, yasta!";
        let message_size_bytes = message_str.as_bytes();
        for addr in &server_addr_v{
            server_socket.send_to(message_size_bytes, &addr).await?;
            println!("Sent: {} to {}", message_str, addr);
        }
        // add my own buffer size to the struct vector
        //value moved here in previous iteration of the loop
        let my_server_struct = ServerInfo {
            address: server_port,
            size: message_str.parse().unwrap(),
        };
        server_buf_size.push(my_server_struct); 


        for addr in &server_addr_v{
            //receive the buffer size from other servers using the same port
            let a = addr;
            println!("Waiting for a reply...");

            let (len, server) = server_socket.recv_from(&mut server_buffer).await?;
            // receive the buffer size from the server as
            let message_server = std::str::from_utf8(&server_buffer[..len])?;
            println!("Received: {} from {}", message_server, server);
            //struct to add to the vector so that we can sort it
            //should be moved outside the loop
            let servers_struct = ServerInfo {
                address: server.to_string(),
                size: message_server.parse().unwrap(),
            };
            //add the struct to the vector
            server_buf_size.push(servers_struct);

       }

       //to update a certain entry in the struct
       //if let Some(index) = people.iter().position(|person| person.name == name_to_update) {
        // Update the age field for the matching person
        // people[index].age = new_age;
    // }
       server_buf_size.sort();

    //    if message_size > server_buf_size[0]{
    //        //execute
    //    }
    //    else if message_size == server_buf_size[0]{
           
    //    }

        // //let least_busy = std::cmp::min(server_addr_v[0], std::cmp::min(b, c));
        // if message_size < server_buf_size[0] && message_size < server_buf_size[1]{
        //     //execute
        // }
        // else if message_size == server_buf_size[0] && message_size == server_buf_size[1]{
            
        // }
        // else if message_size == server_buf_size[0]{

        // }
        // else if message_size == server_buf_size[1]{
            
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

