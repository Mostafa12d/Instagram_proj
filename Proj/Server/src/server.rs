use tokio::net::UdpSocket;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use rand::seq::SliceRandom;

// Struct to represent server information
#[derive(Clone)] // Implement the Clone trait
struct ServerInfo {
    ip: String,
    port: u16,
    status: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // //Specify address form file
    // let filename = "DoSS.txt";
    // let file = File::open(filename).expect("no such file");
    // let buf = BufReader::new(file);
    // let local_addr_v: Vec<String> = buf.lines()
    // .map(|l| l.expect("Could not parse line"))
    // .collect();
    // for addr in &local_addr_v{
    //     println!("{}", addr);
    // }
    //let local_addr = &local_addr_v[0];

    // Specify address from command line
    // let args: Vec<String> = std::env::args().collect();
    // let local_addr = args.get(1).expect("Argument 1 is listening address. Eg: 0.0.0.0:10001");

    // //Specify address from code
    // let local_addr = "0.0.0.0:8086"; // IP address and port you want the server (this process) to listen on

// Create a server (TO BE EDITED LATER TO MAKE THE SERVER DYNAMIC)
    let server_info = ServerInfo {
        ip: "127.0.0.1".to_string(), // Set the server's IP address
        port: 8080, // Set the server's port
        status: 1, // Set the server's status
    };

    // Append server information to a txt file
    append_server_info_to_file(&server_info)?;

    // Start the server
    start_server(&server_info).await?;

    Ok(())
}

// Implement the From_String trait for the ServerInfo struct
impl ServerInfo {
    fn from_string(s: &str) -> ServerInfo {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let ip = parts[1].to_string();
        let port = parts[3].parse().unwrap();
        let status = parts[5].parse().unwrap();

        ServerInfo { ip, port, status }
    }
}

// Append server information to a txt file
fn append_server_info_to_file(info: &ServerInfo) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("DoSS.txt")?;

    // Read the existing contents of the file into a string
    let file_contents = std::fs::read_to_string("DoSS.txt")?;

    // Check if the server info already exists in the file
    if file_contents.contains(&format!("IP: {}, Port: {}", info.ip, info.port)) {
        println!("Server info already exists in the file. Skipping...");
    } else {
        let line = format!("IP: {}, Port: {}, Status: {}\n", info.ip, info.port, info.status);
        file.write_all(line.as_bytes())?;
        println!("Server info added to the file.");
    }

    Ok(())
}



// Start the server
async fn start_server(server_info: &ServerInfo) -> Result<(), Box<dyn Error>> {
    let local_addr = format!("{}:{}", server_info.ip, server_info.port);
    let socket = UdpSocket::bind(&local_addr).await?;

    let mut buffer = [0; 1024];

    // Print server information
    println!("Server is running with the following info:");
    println!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, server_info.status);


    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);

        // Check the server status and respond or delegate the task
        match server_info.status {
            1 => {
                // If status is 1 (active), respond to the client
                // Set the status to busy
                if server_info.status == 1 {
                    set_server_status(&server_info, 2)?;
                    println!("Server is busy.");
                    // Print server information
                    println!("Server is running with the following info:");
                    println!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, server_info.status);


                }

                let response = "Hello, client!";
                let sent_len = socket.send_to(response.as_bytes(), &src).await?;
                println!("Sent: {} bytes to {}", sent_len, src);

                // Set the status back to active
                if server_info.status == 2 {
                    set_server_status(&server_info, 1)?;
                }

                println!("Server is active again.");
            }
            0 => {
                // If status is 0 (inactive), delegate to another active server
                let delegate_server = find_active_server(); // Implement your logic to find an active server
                if let Ok(Some(delegate_server)) = delegate_server {
                    let delegate_addr = format!("{}:{}", delegate_server.ip, delegate_server.port);
                    let response = "Task delegated to another server.";
                    let _ = socket.send_to(response.as_bytes(), &delegate_addr).await;
                    println!("Task delegated to {}.", delegate_addr);
                } else {
                    println!("No active server available to delegate the task.");
                }
            }
            2 => {
                // If status is 2 (busy), log that the server is busy
                println!("Server is busy. No response sent.");
            }
            _ => {
                // Handle other status values as needed
                println!("Invalid server status. No response sent.");
            }
        }
    }

    Ok(())
}

// !!!!!!!!! DOES NOT WORK PROPERLY NEEDS FIXING!!!!!!!!!
fn set_server_status(server_info: &ServerInfo, new_status: u8) -> Result<(), Box<dyn Error>> {
    let file_contents = {
        let file_contents = std::fs::read_to_string("DoSS.txt")?;
        file_contents
    };

    let new_server_info = format!("IP: {}, Port: {}, Status: {}", server_info.ip, server_info.port, new_status);
    let modified_contents = file_contents.replace(&new_server_info, "");

    // let mut file = File::create("DoSS.txt")?;
    // file.write_all(modified_contents.as_bytes())?;

    Ok(())
}


fn find_active_server() -> Result<Option<ServerInfo>, Box<dyn Error>> {
    let file = File::open("DoSS.txt")?;
    let reader = BufReader::new(file);

    let active_servers: Vec<ServerInfo> = reader
        .lines()
        .map(|line| line.expect("Could not parse line"))
        .filter_map(|line| {
            let server_info = ServerInfo::from_string(&line);
            if server_info.status == 1 {
                Some(server_info)
            } else {
                None
            }
        })
        .collect();

    if active_servers.is_empty() {
        Ok(None)
    } else {
        let mut rng = rand::thread_rng();
        if let Some(chosen_server) = active_servers.choose(&mut rng) {
            Ok(Some(chosen_server.clone()))
        } else {
            Err("Failed to select a random server".into())
        }
    }
}
