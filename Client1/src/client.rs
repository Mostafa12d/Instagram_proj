use steganography::encoder::Encoder;
use steganography::util::save_image_buffer;
// This is the client
use tokio::net::UdpSocket;
use tokio::time::interval;
use std::io::BufWriter;
use image::ImageFormat;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageError};
use std::fs::File;
use std::fs;
use std::io::{BufReader, BufRead, Write};
use std::io::Cursor;
use std::io::Read;
use tokio::time::{sleep, Duration};
use steganography::decoder::*;
use steganography::encoder::*;
use std::env;
use std::path::{Path, PathBuf};
use local_ip_address::local_ip;
use std::sync::{Arc, Mutex};



async fn send_servers_multicast(socket: &UdpSocket, message: &[u8], remote_addr1: &str, remote_addr2: &str, remote_addr3: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Send the message to the server
    socket.send_to(message, remote_addr1).await?;
    socket.send_to(message, remote_addr2).await?;
    socket.send_to(message, remote_addr3).await?;
    Ok(())
}
async fn request_ds(socket: &UdpSocket, remote_addr: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Send the message to the server
    let mut rcv_buffer:[u8; 4096] = [0; 4096];
    //message of 1 which is of length 7 bytes meaning request
    let request_buffer = [1; 7];
    let mut text_file = Vec::new();
    socket.send_to(&request_buffer, remote_addr).await?;
    let local_ip = local_ip().unwrap(); // Get the dynamically assigned IP address
    let addr = socket.local_addr()?;
    let port = addr.port();
    let local_addr = local_ip.to_string()+":"+port.to_string().as_str();
    println!("Listening on {}", local_addr);

    let (len, server) = socket.recv_from(&mut rcv_buffer).await?;
    println!("Received {} bytes from {}", len, server);
    let message_server = std::str::from_utf8(&rcv_buffer[..len])?;
    //add received server to the vector
    for line in message_server.lines(){
    if !text_file.contains(&line.to_string()) {
        //add all the ips except the one that sent the message
        if line != local_addr{
        text_file.push(line.to_string());
        println!("Received this address: {} ", line);
        }
    } 
    }
    Ok(text_file)
}

async fn send_to_peer(socket: &UdpSocket, remote_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Send a request to the client
    let peer_request_buffer = [1; 6];
    socket.send_to(&peer_request_buffer, remote_addr).await?;
    Ok(())
}
async fn server_request(socket: &UdpSocket, remote_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Send a request to the server
    let server_request_buffer = [1; 5];
    socket.send_to(&server_request_buffer, remote_addr).await?;
    Ok(())
}

fn resize_image(input_path: &str, output_path: &str, new_width: u32) -> Result<(), ImageError> {
    // Load the image
    let img = image::open(input_path)?;

    // Get the current dimensions of the image
    let (width, height) = img.dimensions();

    // Calculate the new height while maintaining aspect ratio
    let aspect_ratio = height as f32 / width as f32;
    let new_height = (new_width as f32 * aspect_ratio).round() as u32;

    // Resize the image
    let resized_img = img.resize_exact(new_width, new_height, FilterType::Nearest);

    // Save the resized image
    resized_img.save(output_path)?;

    Ok(())
}

fn resize_all_images(new_width: u32) -> Result<(), ImageError> {
    let imgs_directory = Path::new("./decoded_imgs");
    let resized_directory = Path::new("./resized_imgs");

    // Create the resized_imgs directory if it doesn't exist
    if !resized_directory.exists() {
        fs::create_dir(resized_directory)?;
    }

    for entry in fs::read_dir(imgs_directory)? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a file and has an image extension
        if path.is_file() && is_image_file(&path) {
            let input_path = path.to_str().unwrap();

            // Adjust the output path to save in the resized_imgs folder
            let mut output_path = PathBuf::from(resized_directory);
            output_path.push(format!("resized_{}", path.file_name().unwrap().to_str().unwrap()));

            resize_image(input_path, output_path.to_str().unwrap(), new_width)?;
        }
    }

    Ok(())
}

// Helper function to determine if a path is an image file
fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => true,
            _ => false,
        },
        None => false,
    }
}

async fn receive_image(folder: &String, image_string: &String ,  socket: &UdpSocket) -> Result<String, Box<dyn std::error::Error>> {
    // Send a request to the server
    let mut server_buffer = [0; 4096];
    // let mut received_data = Vec::new();
    let image_name = folder.to_string() + "/img_rcv" + &image_string + ".png";
    let image_cloned = image_name.clone();
    let mut file = File::create(image_name)?;
    let mut i = 0;
    loop{
        i+=1;
        // println!("Waiting for a message...");
        //receive message from client
        server_buffer = [0; 4096];
        
        let (len, server) = socket.recv_from(&mut server_buffer).await?;

        println!("Received {} bytes from {}", len, server);
        file.write_all(&server_buffer[..len])?;
        server_buffer = [0; 4096];
        // println!("Received string: {}", client);
        // breah after the last packet
        if len != 4096 {
            break;
        }
    }
    Ok(image_cloned)
}

async fn user_menu(socket: Arc<Mutex<UdpSocket>>) {
    loop {
        println!("===== User Menu =====");
        println!("1. Option 1");
        println!("2. Option 2");
        println!("3. Option 3");
        println!("0. Exit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read user input");
        let choice: u32 = input.trim().parse().unwrap_or_default();

        match choice {
            1 => {
                // Handle Option 1
                println!("Selected Option 1");
            }
            2 => {
                // Handle Option 2
                println!("Selected Option 2");
            }
            3 => {
                // Handle Option 3
                println!("Selected Option 3");
            }
            0 => {
                // Exit the menu
                println!("Exiting User Menu");
                break;
            }
            _ => {
                println!("Invalid choice. Please enter a valid option.");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let repetition_count: usize = args.get(1) // Get the second element (index 1)
        .expect("Please provide a repetition count as the first argument")
        .parse() // Attempt to parse the argument as an integer
        .expect("Please provide a valid integer for the repetition count");


    // for all images in the imgs folder, call the resize_image function
    resize_all_images(50)?;

    //original
    let remote_addr1 = "10.40.35.23:10014"; // IP address and port of the Server 0
    let remote_addr2 = "10.40.35.23:10015"; // IP address and port of the Server 1
    let remote_addr3 = "10.40.35.23:10016"; // IP address and port of the Server 2

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    // get my port number
    // let local_addr = socket.local_addr()?;
    // let port = local_addr.port();
    // println!("Listening on {}", port);
    
    let local_ip = local_ip().unwrap(); // Get the dynamically assigned IP address
    let addr = socket.local_addr()?;
    let port = addr.port();
    let local_addr = local_ip.to_string()+":"+port.to_string().as_str();
    println!("Listening on {}", local_addr);

    // Spawn the user menu as a separate asynchronous task
    let socket2 = Arc::new(Mutex::new(UdpSocket::bind("0.0.0.0:0").await?));
    let socket_clone = Arc::clone(&socket2);
    tokio::spawn(user_menu(socket_clone));
    
    tokio::spawn(async move {
        //loop{
            let socket2 = Arc::new(Mutex::new(UdpSocket::bind("0.0.0.0:0").await));  
            let socket_clone2 = Arc::clone(&socket2);
            let result = socket_clone2.lock().unwrap();
            let udp_socket = match &*result {
                Ok(socket) => socket,
                Err(err) => {
                    // Handle the error, you might want to log it or return an error
                    eprintln!("Error accessing UdpSocket: {:?}", err);
                    return;
                }
            };
            println!("Waiting for a message...");
            let mut server_buffer: [u8; 4096] = [0; 4096]; // this is to receive from the servers with
            let mut ping_buffer: [u8; 8] = [0; 8]; // Tells the server I'm up
            //function to send an image
            let image_path = "./src/car.png";
            let mut img = File::open(image_path).unwrap();
            let mut buffer = Vec::new();
            // println!("Image Buffer content: {:?}", buffer);
            // iterate 400 times in a for loop
            img.read_to_end(&mut buffer).unwrap();    
            let mut image_num:u32 = 0;

            // ping servers "I'm up"
            send_servers_multicast(&udp_socket, &ping_buffer, remote_addr1, remote_addr2, remote_addr3).await.unwrap();
            //let mut client_vec = Vec::new();
            println!("wa2ef henaaa");
            //client_vec = request_ds(&socket, remote_addr1).await.unwrap();
            println!("wa2ef henaaa2");
            let request_buffer = [1; 5];
            //send image to servers
            for i in 0..repetition_count { 
                println!("wa2ef henaaa");   
                send_servers_multicast(&udp_socket, &request_buffer, remote_addr1, remote_addr2, remote_addr3).await.unwrap();
                println!("wa2ef henaaa++");   
                let mut sequence_number:u64 = 1;

                let (len, serv) = udp_socket.recv_from(&mut server_buffer).await.unwrap();

                //let (len, serv) = cloned_socket2.lock().unwrap().recv_from(&mut server_buffer).await.unwrap();
                println!("wa2ef henaaa");   
                if len == 4{
                    for chunk in buffer.chunks(4096) {
                        let mut packet_vector: Vec<u8> = Vec::new();
                        
                        // Include the sequence number in the packet
                        packet_vector.extend_from_slice(&sequence_number.to_be_bytes());
                        packet_vector.extend_from_slice(chunk);
        
                        //send packets to server
                        //println!("Sending chunk of: {}", chunk.len());
                        udp_socket.send_to(&packet_vector, serv).await.unwrap();
                        // socket.send_to(&packet_vector, remote_addr2).await.unwrap();
                        // socket.send_to(&packet_vector, remote_addr3).await.unwrap();
                        
                        //sleep for 1ms
                        sleep(Duration::from_millis(100)).await;
                        // Increment the sequence number for the next packet
                        sequence_number += 1;
                        println!("Sent packet of size {}"  , packet_vector.len());
                    }
        
                    println!("Sent the image to the servers");
                    let folder = "server_imgs".to_string();
                    if !Path::new(&folder).exists() {
                        fs::create_dir(&folder).unwrap();
                    }
                    let image_string = image_num.to_string();
                    
                    let image_cloned =  receive_image(&folder, &image_string, &udp_socket).await.unwrap();            
        
                    // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());
                    //if image_num == 0 {
                    let image_name2 = "decoded_imgs/decoded_img".to_string() + &image_string + ".png";
                    // let mut file2 = File::create(image_name2)?;
                    let clone = image::open(image_cloned).unwrap();
                    let img_buffer = clone.to_rgba();
                    // println!("Image Buffer content: {:?}", img_buffer);
                    //let img_buffer_clone = img_buffer.clone();
                    let decoded_image = Decoder::new(img_buffer);
                    let decoded_secret = decoded_image.decode_alpha();
        
                    let decoded_img = image::load_from_memory(&decoded_secret).unwrap();
                    let mut output_file = BufWriter::new(File::create(image_name2).unwrap());
                    decoded_img.write_to(&mut output_file, ImageFormat::PNG).unwrap();
                    // file2.write_all(&decoded_secret).unwrap();
                    //}
                    image_num += 1;
                }
            }
        
            // if client_vec.len() != 0 {
            //     //println!("Received the address: {} from server", &client_vec[0]);
            //     let clienttt="172.29.255.134:12345";
            //     send_to_peer(&socket, &clienttt).await.unwrap();
            //     //receive from client low res images 
            //     let folder = "client_imgs".to_string();
            //     if !Path::new(&folder).exists() {
            //         fs::create_dir(&folder).unwrap();
            //     }
            //     let image_num = 0;  
            //     let image_string = image_num.to_string();
            //     let trial = receive_image(&folder, &image_string, &socket).await.unwrap();
            // }
            
            let mut client_buffer = [1; 6];
            // rceive frmo cleint length of 6 means a client requesting all low res images
            let (len, src) = udp_socket.recv_from(&mut client_buffer).await.unwrap();
            if len == 6 {
                // send all the low res images to the client
                let imgs_directory = Path::new("./resized_imgs");
                for entry in fs::read_dir(imgs_directory).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
        
                    // Check if the entry is a file and has an image extension
                    if path.is_file() && is_image_file(&path) {
                        let input_path = path.to_str().unwrap();
                        let mut img = File::open(input_path).unwrap();
                        let mut buffer = Vec::new();
                        img.read_to_end(&mut buffer).unwrap();
                        //send image to client
                        udp_socket.send_to(&buffer, src).await.unwrap();
                    }
                }
                //clear buffer
                client_buffer = [1; 6];
            }
            // loop {
            //     println!("Waiting for a message...");
                    
            // }
        //}
    });
    

    
    // receive low res images from clients
    //sending a request for a server to establish connection. Message code len of 5
    
        
    //add 2 secs delay
    //sleep(Duration::from_secs(2)).await;
//     loop {
//         println!("Waiting for a message...");
//         let (len, src) = socket.recv_from(&mut message_buffer).await?;
//         let message = std::str::from_utf8(&message_buffer[..len])?;
//         println!("Received: {} from {}", message, src);

//     }

    Ok(())
}