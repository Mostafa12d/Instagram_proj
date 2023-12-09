// This is the client
use steganography::encoder::Encoder;
use steganography::util::*;
// This is the client
use tokio::net::UdpSocket;
use tokio::time::interval;
use core::num;
use std::io::BufWriter;
use std::process::ExitCode;
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
use tokio::task;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use minifb::{Window, WindowOptions};
use std::time::{Instant};
use tokio::time::timeout;


pub fn display_image(mut img: DynamicImage) {
    let fixed_width = 800; // Fixed width for display
    let fixed_height = 600; // Fixed height for display

    // Get the current dimensions of the image
    let (width, height) = img.dimensions();

    // Calculate new dimensions to maintain aspect ratio
    let aspect_ratio = width as f32 / height as f32;
    let (new_width, new_height) = if aspect_ratio > 1.0 {
        // Image is wider than tall, fix width and scale height
        (fixed_width, (fixed_width as f32 / aspect_ratio) as u32)
    } else {
        // Image is taller than wide, fix height and scale width
        ((fixed_height as f32 * aspect_ratio) as u32, fixed_height)
    };

    // Resize the image
    img = img.resize_exact(new_width, new_height, FilterType::Nearest);

    let buffer: Vec<u32> = img.to_rgba().into_raw().chunks(4).map(|c| {
        ((c[3] as u32) << 24) | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)
    }).collect();

    let mut window = Window::new(
        "Image Display",
        new_width as usize,
        new_height as usize,
        WindowOptions::default()
    ).expect("Unable to open window");

    let start_time = Instant::now();
    let duration = Duration::new(8, 0); // 8 seconds

    while window.is_open() {
        if Instant::now() - start_time >= duration {
            break; // Break the loop after 8 seconds
        }

        window.update_with_buffer(&buffer, new_width as usize, new_height as usize)
              .expect("Failed to update window");
    }
}


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
    //println!("Listening on {}", local_addr);

    let (len, server) = socket.recv_from(&mut rcv_buffer).await?;
    //println!("Received {} bytes from {}", len, server);
    let message_server = std::str::from_utf8(&rcv_buffer[..len])?;
    let mut i = 0;
    //add received server to the vector
    for line in message_server.lines(){
    if !text_file.contains(&line.to_string()) {
        //add all the ips except the one that sent the message
        if line != local_addr{
        text_file.push(line.to_string());
        i+=1;
        //println!("Received this address: {} ", line);
        println!("Client {}: {} ",i, line)
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
    let imgs_directory = Path::new("./my_imgs");
    let resized_directory = Path::new("./my_low_res_imgs");

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
        //println!("Waiting for a message...");
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

fn print_stats(total_images: u32, total_data_sent: u64, elapsed_time: Duration) {
    let elapsed_secs = elapsed_time.as_secs_f64(); // Converts Duration to seconds as a float

    println!("Statistics:");
    println!("Total images processed: {}", total_images);
    println!("Total data sent: {} bytes", total_data_sent);
    println!("Total time elapsed: {:.2} seconds", elapsed_secs);
    println!("Average data rate: {:.2} bytes/second", total_data_sent as f64 / elapsed_secs);
    println!("Average time per image: {:.2} seconds", elapsed_secs / total_images as f64);
}

fn user_menu(shared_data: Arc<Mutex<SharedData>>) {
    loop {
        println!("Please select an option:");
        println!("1. Request list of Available Clients");
        println!("2. Request low-resolution image from a client");
        println!("3. Request image from a client");
        println!("4. Encrypt Image through server");
        println!("5. Send image to client");
        println!("6. View available decoded images");
        println!("7. View available low-res images");
        println!("8. Exit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");

        let mut data = shared_data.lock().unwrap();
        match input.trim() {
            "1" => data.option = 1,
            "2" => {
                println!("Please enter the number of the client you want to request from:");
                let mut additional_info = String::new();
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read additional information");
                data.option = 2;
                data.additional_input = additional_info.trim().to_string();
            },            
            "3" => data.option = 3,
            "4" => {
                println!("How many images would you like to encrypt?:");
                let mut additional_info = String::new();
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read additional information");
                data.option = 4;
                data.additional_input = additional_info.trim().to_string();
            },
            "5" => {
                println!("Please enter the number of the client you want to send to:");
                let mut additional_info = String::new();
                let mut img_views = String::new();
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read additional information");
                println!("Please enter the number of allowed views:");
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read img views");
                data.option = 5;
                data.additional_input = additional_info.trim().to_string();
                data.img_views = img_views.trim().to_string();
            },
            "6" => {
                println!("Please enter the number of the decoded image you want to see:");
                let mut additional_info = String::new();
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read additional information");
                data.option = 6;
                data.additional_input = additional_info.trim().to_string();
            },
            "7" => {
                println!("Please enter the number of the low-res image you want to see:");
                let mut additional_info = String::new();
                std::io::stdin().read_line(&mut additional_info).expect("Failed to read additional information");
                data.option = 7;
                data.additional_input = additional_info.trim().to_string();
            },
            "8" => data.option = 8,
            _ => {
                println!("Invalid option, please try again.");
                continue;
            },
        }
    }
}


fn delete_all_files_in_directory(dir: &str) -> std::io::Result<()> {
    let path = Path::new(dir);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}

struct SharedData {
    option: i32,
    additional_input: String,
    img_views: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    
    //original
    let remote_addr1 = "192.168.1.9:10014"; // IP address and port of the Server 0
    let remote_addr2 = "192.168.1.9:10015"; // IP address and port of the Server 1
    let remote_addr3 = "192.168.1.9:10016"; // IP address and port of the Server 2
    
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
    
    let mut server_buffer: [u8; 4096] = [0; 4096]; // this is to receive from the servers with
    let mut ping_buffer: [u8; 8] = [0; 8]; // Tells the server I'm up
    //function to send an image
    let image_path = "./my_imgs/car.png";
    let mut img = File::open(image_path)?;
    let mut buffer = Vec::new();
    // println!("Image Buffer content: {:?}", buffer);
    // iterate 400 times in a for loop
    img.read_to_end(&mut buffer)?;    
    let mut image_num:u32 = 0;
    
    let (tx, mut rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel(32);
    let tx1 = tx.clone();
    
    // create a thread to do the user_menu 
    
    //let shared_data = Arc::new(Mutex::new(0));
    let shared_data = Arc::new(Mutex::new(SharedData { option: 0, additional_input: String::new(), img_views: String::new() }));

    // Clone the Arc to pass to the thread
    let shared_data_clone = Arc::clone(&shared_data);
    // ping servers "I'm up"
    send_servers_multicast(&socket, &ping_buffer, remote_addr1, remote_addr2, remote_addr3).await?;
    
    // Spawn the user_menu in a separate thread
    thread::spawn(move || {
        user_menu(shared_data_clone);
    });
    
    resize_all_images(50)?;
    
    let mut client_vec = Vec::new();
    loop {
        // Lock the mutex and read the data
        let mut data = shared_data.lock().unwrap();
        
        match data.option {
            1 => {
                println!("Option 1 selected: Request list of Available Clients");
                // Request list of available clients from servers
                client_vec = request_ds(&socket, remote_addr1).await?;
                if client_vec.len() == 0 {
                    println!("No clients available");
                }
                data.option = 0; // Reset the shared data after processing
            },
            2 => { //needs bug fix
                println!("Option 2 selected: Request low-resolution image from a client");
                //println!("Received the address: {} from server", &client_vec[0]);

                if client_vec.len() != 0 {
                    //println!("Received the address: {} from server", &client_vec[0]);
                    
                    
                    // Request low res images from a peer
                    //let clienttt="172.29.255.134:12345";
                    let clienttt = &client_vec[&data.additional_input.parse::<usize>().unwrap() - 1];
                    println!("Sending request to client: {}", clienttt);
                    send_to_peer(&socket, &clienttt).await?;
                    
                    
                    
                    //receive from client low res images 
                    // let folder = "client_imgs".to_string();
                    //     if !Path::new(&folder).exists() {
                        //         fs::create_dir(&folder)?;
                        //     }
                        let folder = "rcvd_low_res_imgs".to_string();
                        let image_string = image_num.to_string();
                    println!("Receiving image from client: {}", clienttt);
                    
                    
                    //let trial = receive_image(&folder, &image_string, &socket).await?;
                    let timeout_duration = Duration::from_secs(1);
                    let mut i = 0;
                    loop {
                        let image_string = image_num.to_string();
                        let receive_result = timeout(timeout_duration, receive_image(&folder, &image_string, &socket)).await;
                    
                        match receive_result {
                            Ok(Ok(image_cloned)) => {
                                println!("Received image from client: {}", clienttt);
                                // ... handle the received image ...
                                image_num += 1; // Increment to prepare for the next image
                            },
                            Ok(Err(e)) => {
                                println!("Failed to receive image: {}", e);
                                break; // Stop receiving further images on error
                            },
                            Err(_) => {
                               // println!("Timeout occurred while receiving the image");
                                break; // Stop receiving further images on timeout
                            }
                        }
                        i+=1;
                    }
                    
                    println!("Received {} low-res images from client", i);

                    //println!("Received image from client: {}", clienttt);
                    image_num += 1;  
                }
                    data.option = 0; // Reset the shared data after processing
                    data.additional_input.clear();                    
            },
            3 => {
                println!("Option 3 selected: Request image from a client");
                for client in client_vec.iter() {
                    println!("Client: {}", client);
                }
                // Implement logic for option 3
                data.option = 0; // Reset the shared data after processing
            },
            4 => {
                
                println!("Option 4 selected: Encrypt Images through server");
                let request_buffer: [u8; 5] = [1; 5];
                //send image to servers
                // create an int with value of data.additional_input
                let additional_input = data.additional_input.parse::<u32>().unwrap();                
                let mut image_num:u32 = 0;
                let mut total_data_sent: u64 = 0;
                let start_time = Instant::now();
                for i in 0..additional_input {    
                    send_servers_multicast(&socket, &request_buffer, remote_addr1, remote_addr2, remote_addr3).await?;
                    let mut sequence_number:u64 = 1;
                    let (len, serv) = socket.recv_from(&mut server_buffer).await?; 
                    // if receievd a notification from the elected leader, send them the image for encryption
                    if len == 4{
                        // socket.connect(&serv).await?;
                        for chunk in buffer.chunks(4096) {
                            let mut packet_vector: Vec<u8> = Vec::new();
                            
                            // Include the sequence number in the packet
                            packet_vector.extend_from_slice(&sequence_number.to_be_bytes());
                            packet_vector.extend_from_slice(chunk);
                            
                            //send packets to server
                            //println!("Sending chunk of: {}", chunk.len());
                            socket.send_to(&packet_vector, serv).await?;
                            //socket.send(&packet_vector).await?;
                            
                            //sleep for 1ms
                            sleep(Duration::from_millis(100)).await;
                            // Increment the sequence number for the next packet
                            sequence_number += 1;
                            total_data_sent += packet_vector.len() as u64;
                            println!("Sent packet of size {}"  , packet_vector.len());
                        }
                        
                        println!("Sent the image to the servers");
                        let folder = "server_imgs".to_string();
                        if !Path::new(&folder).exists() {
                            fs::create_dir(&folder)?;
                        }
                        let image_string = image_num.to_string();
                        
                        
                        // RECEIVE IMAGES FROM SERVERS
                        let image_cloned =  receive_image(&folder, &image_string, &socket).await?;            
                        // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());
                        //if image_num == 0 {
                        let image_name2 = "decoded_imgs/decoded_img".to_string() + &image_string + ".png";
                        // let mut file2 = File::create(image_name2)?;
                        let clone = image::open(image_cloned)?;
                        let img_buffer = clone.to_rgba();
                        // println!("Image Buffer content: {:?}", img_buffer);
                        //let img_buffer_clone = img_buffer.clone();
                        let decoded_image = Decoder::new(img_buffer);
                        let decoded_secret = decoded_image.decode_alpha();

                        
                        let decoded_img = image::load_from_memory(&decoded_secret)?;
                        let mut output_file = BufWriter::new(File::create(image_name2)?);
                        decoded_img.write_to(&mut output_file, ImageFormat::PNG)?;
                        
                        // // Serialize text
                        // let text = "Hello, world!".to_string();
                        // let text_bytes = str_to_bytes(&text);

                        // // Embed the serialized text into the primary image
                        // let encoder = Encoder::new(&text_bytes, decoded_img);
                        // let encoded_image = encoder.encode_alpha();
                        // // Save the encoded image
                        // // encoded_image.save("encoded_text_image.png")?;
                        // save_image_buffer(encoded_image.clone(), "./src/encoded_txt.png".to_string());

                        // // Extract the embedded data
                        // let decoder = Decoder::new(encoded_image);
                        // let decoded_bytes = decoder.decode_alpha();
                        // let clean_buffer: Vec<u8> = decoded_bytes.into_iter()
                        //             .filter(|b| {
                        //                 *b != 0xff_u8
                        //             })
                        //             .collect();
                        // //Convert those bytes into a string we can read
                        // let message = bytes_to_str(clean_buffer.as_slice());
                        // //Print it out!
                        // println!("{:?}", message);
                        image_num += 1;
                    }
                }
                let elapsed_time = start_time.elapsed();
                print_stats(additional_input, total_data_sent, elapsed_time);
                data.option = 0; // Reset the shared data after processing
                data.additional_input.clear();
            },
            5 => {
                println!("Option 5 selected: Send image to client");
                let request_buffer: [u8; 9] = [1; 9];
                /////
                // read the image from the file
                let num_views = &data.img_views;
                let image_path = "./decoded_imgs/decoded_img0.png";
                // Serialize text ///////////////
                let decoded_img = image::open(image_path)?;

                // let text = "Hello, world!".to_string();
                let text_bytes = str_to_bytes(&num_views);

                // Embed the serialized text into the primary image
                let encoder = Encoder::new(&text_bytes, decoded_img);
                let encoded_image = encoder.encode_alpha();
                // Save the encoded image
                save_image_buffer(encoded_image.clone(), "./encoded/encoded_txt.png".to_string());
                ////
                // open the encoded file
                let image_path = "./encoded/encoded_txt.png";
                let mut en_img = File::open(image_path)?;
                let mut encoded_vec = Vec::new();  
                en_img.read_to_end(&mut encoded_vec).unwrap();
                // img.read_to_end(&mut buffer)?;
                socket.send_to(&request_buffer, &client_vec[&data.additional_input.parse::<usize>().unwrap() - 1]).await?;
                for chunk in encoded_vec.chunks(4096) {
                        let mut packet_vector: Vec<u8> = Vec::new();
                        
                        // Include the sequence number in the packet
                    //    packet_vector.extend_from_slice(&sequence_number.to_be_bytes());
                        packet_vector.extend_from_slice(chunk);
        
                        //send packets to server
                        //println!("Sending chunk of: {}", chunk.len());
                        socket.send_to(&packet_vector, &client_vec[0]).await?;
                        //socket.send(&packet_vector).await?;
                        
                        //sleep for 1ms
                        sleep(Duration::from_millis(100)).await;
                        // Increment the sequence number for the next packet
                        //sequence_number += 1;
                        println!("Sent packet of size {}"  , packet_vector.len());
                    }
        
                    data.option = 0; // Reset the shared data after processing
                    data.additional_input.clear();                            
    
            },
            6 => { //need to update directory and file name of images to be displayed
                let mut image_view = 2; // need to handle this somewhere else
                println!("Option  6: View available decoded images");
                if image_view != 0 {
                    let file_path = format!("./rcvd_client_imgs/img_rcv{}.png", data.additional_input.parse::<usize>().unwrap() - 1);
                    match image::open(&file_path) {
                    Ok(img) => display_image(img),
                    Err(e) => println!("Failed to open image: {}", e),
                                             }
                                 }
                image_view -= 1;
                println!("Image views left: {}", image_view);
                    data.option = 0; // Reset the shared data after processing
                    data.additional_input.clear();                    
            },
            7 => { 
                println!("Option  7: View available low-res images");
                
                let file_path = format!("./rcvd_low_res_imgs/img_rcv{}.png", data.additional_input.parse::<usize>().unwrap() - 1);
                match image::open(&file_path) {
                    Ok(img) => display_image(img),
                    Err(e) => println!("Failed to open image: {}", e),
                }
                    data.option = 0; // Reset the shared data after processing
                    data.additional_input.clear();                    
            },
            8 => {
                println!("Exiting...");
    
                // Call the function to delete all files in 'client_imgs'
                if let Err(e) = delete_all_files_in_directory("rcvd_client_imgs") {
                    eprintln!("Failed to delete files: {}", e);
                }
                if let Err(e) = delete_all_files_in_directory("rcvd_low_res_imgs") {
                    eprintln!("Failed to delete files: {}", e);
                }
                if let Err(e) = delete_all_files_in_directory("server_imgs") {
                    eprintln!("Failed to delete files: {}", e);
                }
                if let Err(e) = delete_all_files_in_directory("decoded_imgs") {
                    eprintln!("Failed to delete files: {}", e);
                }
                if let Err(e) = delete_all_files_in_directory("my_low_res_imgs") {
                    eprintln!("Failed to delete files: {}", e);
                }
    
                // Implement any additional cleanup or exit logic
                break;
            },
            0 => {
                // println!("Helloooo");
                // println!("default: {}", *data);
                //*data = 0; // Reset the shared data if invalid input is received
                let mut length = 0;
                let mut source;
                    let mut client_buffer = [0; 4096];
                    // rceive frmo cleint length of 6 means a client requesting all low res images
                    match socket.try_recv_from(&mut client_buffer) {
                        Ok((len, src)) => {
                            length = len.clone();
                            source = src;
                            // Handle the case where data is successfully received
                            println!("Received {} bytes from {}", len, src);
                            // Process the data...
                            if length == 6 {
                                // send all the low res images to the client
                                let imgs_directory = Path::new("./my_low_res_imgs");
                                for entry in fs::read_dir(imgs_directory)? {
                                    let entry = entry?;
                                    let path = entry.path();
                        
                                    // Check if the entry is a file and has an image extension
                                    if path.is_file() && is_image_file(&path) {
                                        let input_path = path.to_str().unwrap();
                                        let mut img = File::open(input_path)?;
                                        let mut buffer = Vec::new();
                                        img.read_to_end(&mut buffer)?;
                                        //send image to client
                    
                                        for chunk in buffer.chunks(4096) {
                                            let mut packet_vector: Vec<u8> = Vec::new();
                                            
                                            // Include the sequence number in the packet
                                                //packet_vector.extend_from_slice(&sequence_number.to_be_bytes());
                                                packet_vector.extend_from_slice(chunk);
                                
                                                //send packets to server
                                                //println!("Sending chunk of: {}", chunk.len());
                                                socket.send_to(&packet_vector, source).await?;
                                                //socket.send(&packet_vector).await?;
                                                
                                                //sleep for 1ms
                                                sleep(Duration::from_millis(100)).await;
                                                // Increment the sequence number for the next packet
                                                //sequence_number += 1;
                                                println!("Sent packet of size {}"  , packet_vector.len());
                                            }
                                    }
                                }
                                println!("Sent all the low res images to the client");
                                //clear buffer
                                client_buffer = [0; 4096];
                            }
                        if length != 6{
                        let image_string = image_num.to_string();
                
                        let folder = "rcvd_client_imgs".to_string();
                        // RECEIVE IMAGES FROM SERVERS
                        let image_cloned =  receive_image(&folder, &image_string, &socket).await?;
                        // Extract the embedded data
                        let decoded_img = image::open(image_cloned)?;
                        let decoded_img_buffer = decoded_img.to_rgba();
                        let decoder = Decoder::new(decoded_img_buffer);
                        let decoded_bytes = decoder.decode_alpha();
                        let clean_buffer: Vec<u8> = decoded_bytes.into_iter()
                                    .filter(|b| {
                                        *b != 0xff_u8
                                    })
                                    .collect();
                        //Convert those bytes into a string we can read
                        let message = bytes_to_str(clean_buffer.as_slice());
                        //Print it out!
                        println!("{:?}", message);        
                        // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());
                        //if image_num == 0 {
                        // let image_name2 = "decoded_imgs/decoded_img".to_string() + &image_string + ".png";
                        // // let mut file2 = File::create(image_name2)?;
                        // let clone = image::open(image_cloned)?;
                        // let img_buffer = clone.to_rgba();
                        // // println!("Image Buffer content: {:?}", img_buffer);
                        // //let img_buffer_clone = img_buffer.clone();
                        // let decoded_image = Decoder::new(img_buffer);
                        // let decoded_secret = decoded_image.decode_alpha();
                        
                        // let decoded_img = image::load_from_memory(&decoded_secret)?;
                        // let mut output_file = BufWriter::new(File::create(image_name2)?);
                        // decoded_img.write_to(&mut output_file, ImageFormat::PNG)?;
                        // file2.write_all(&decoded_secret).unwrap();
                        //}
                        image_num += 1;
                        }
                        },
                        Err(e) => {
                            // Handle the error case
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                // This is the expected "non-blocking" behavior when there's no data
                                // println!("No data available to read");
                            } else {
                                // Handle other kinds of errors
                                eprintln!("Error occurred: {}", e);
                            }
                        }
                    }
                                        //println!("recieved from client: {}", src);
            }, // No new input, do nothing
            _ => {
                    // *data=0;
                },
        }

        // Release the lock before sleeping
        drop(data);

        // Sleep for a short duration to reduce CPU usage
        thread::sleep(Duration::from_millis(100));
    }
    


    // Request list of available clients from servers
    // client_vec = request_ds(&socket, remote_addr1).await?;

    

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