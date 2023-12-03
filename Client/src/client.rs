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
use std::io::{BufReader, BufRead, Write};
use std::io::Cursor;
use std::io::Read;
use tokio::time::{sleep, Duration};
use steganography::decoder::*;
use steganography::encoder::*;
use std::env;


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

    let (len, server) = socket.recv_from(&mut rcv_buffer).await?;
    println!("Received {} bytes from {}", len, server);
    let message_server = std::str::from_utf8(&rcv_buffer[..len])?;
    println!("Received the address: {} from server {}", message_server, server);
    //add received server to the vector
    if !text_file.contains(&message_server.to_string()) {
        text_file.push(message_server.to_string());
    } 
    Ok(text_file)
}

async fn send_to_peer(socket: &UdpSocket, remote_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Send a request to the client
    let peer_request_buffer = [1; 6];
    socket.send_to(&peer_request_buffer, remote_addr).await?;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let repetition_count: usize = args.get(1) // Get the second element (index 1)
        .expect("Please provide a repetition count as the first argument")
        .parse() // Attempt to parse the argument as an integer
        .expect("Please provide a valid integer for the repetition count");

    resize_image("./src/car.png", "./src/resized.png", 50)?;

    //original
    let remote_addr1 = "172.29.255.134:10014"; // IP address and port of the Server 0
    let remote_addr2 = "172.29.255.134:10015"; // IP address and port of the Server 1
    let remote_addr3 = "172.29.255.134:10016"; // IP address and port of the Server 2

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut server_buffer = [0; 4096]; // this is to receive from the servers with
    let mut ping_buffer: [u8; 8] = [0; 8]; // Tells the server I'm up
    //function to send an image
    let image_path = "./src/car.png";
    let mut img = File::open(image_path)?;
    let mut buffer = Vec::new();
    // println!("Image Buffer content: {:?}", buffer);
    // iterate 400 times in a for loop
    img.read_to_end(&mut buffer)?;    
    let mut image_num:u32 = 0;

    // ping servers "I'm up"
    send_servers_multicast(&socket, &ping_buffer, remote_addr1, remote_addr2, remote_addr3).await?;
    request_ds(&socket, remote_addr1).await?;

   //send image to servers
   for i in 0..repetition_count {    
       let mut sequence_number:u64 = 1;
       
       for chunk in buffer.chunks(4096) {
            let mut packet_vector: Vec<u8> = Vec::new();

            // Include the sequence number in the packet
            packet_vector.extend_from_slice(&sequence_number.to_be_bytes());
            packet_vector.extend_from_slice(chunk);

            //send packets to server
            //println!("Sending chunk of: {}", chunk.len());
            socket.send_to(&packet_vector, remote_addr1).await?;
            socket.send_to(&packet_vector, remote_addr2).await?;
            socket.send_to(&packet_vector, remote_addr3).await?;
            
            //sleep for 1ms
            sleep(Duration::from_millis(100)).await;
            // Increment the sequence number for the next packet
            sequence_number += 1;
            println!("Sent packet of size {}"  , packet_vector.len());
        }
    
        println!("Sent the image to the servers");
    
        server_buffer = [0; 4096];
        // let mut received_data = Vec::new();
        let image_string = image_num.to_string();
        let image_name = "imgs/img_rcv".to_string() + &image_string + ".png";
        let image_cloned = image_name.clone();
        let mut file = File::create(image_name)?;
        let mut i = 0;
        loop{
            i+=1;
            // println!("Waiting for a message...");
            //receive message from client
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
        

        // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());
        //if image_num == 0 {
        let image_name2 = "imgs/decoded_img".to_string() + &image_string + ".png";
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
        // file2.write_all(&decoded_secret).unwrap();
        //}
        image_num += 1;
    }

        
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