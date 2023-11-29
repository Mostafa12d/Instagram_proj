use steganography::encoder::Encoder;
use steganography::util::save_image_buffer;
// This is the client
use tokio::net::UdpSocket;
use std::io::BufWriter;
use image::ImageFormat;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::io::Cursor;
use std::io::Read;
use tokio::time::{sleep, Duration};
use steganography::decoder::*;
use steganography::encoder::*;
use image::DynamicImage;
use std::env;

// fn main(){

//     let cover = image::open("./src/bambo.jpg").unwrap();
//     let mut secret_image = Vec::new();
//     let mut secret = File::open("./src/yaboy.jpg").unwrap();
//     secret.read_to_end(&mut secret_image).unwrap();

    // let encoders = Encoder::new(&secret_image, cover);

//     let encoded_image = encoders.encode_alpha();
//     save_image_buffer(encoded_image.clone(), "./src/encoded.jpg".to_string());
//    let clone = encoded_image.clone();

//     let mut file = File::create("./src/decoded.jpg".to_string()).unwrap();
//     let decoded_image = Decoder::new(clone);
//     let decoded_secret = decoded_image.decode_alpha();
//     file.write_all(&decoded_secret);

//     save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());

// }

// fn parse_nack_message(nack_msg: &[u8]) -> Vec<u64> {
//     // Implement parsing logic here
//     // Example: Convert each 8 bytes to a u64 sequence number
//     nack_msg.chunks(8).map(|chunk| {
//         let (int_bytes, _) = chunk.split_at(std::mem::size_of::<u64>());
//         u64::from_be_bytes(int_bytes.try_into().unwrap())
//     }).collect()
// }
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let repetition_count: usize = args.get(1) // Get the second element (index 1)
        .expect("Please provide a repetition count as the first argument")
        .parse() // Attempt to parse the argument as an integer
        .expect("Please provide a valid integer for the repetition count");
    //original

    //original
    let remote_addr1 = "10.40.55.192:10014"; // IP address and port of the Server 0
    let remote_addr2 = "10.40.55.192:10015"; // IP address and port of the Server 1
    let remote_addr3 = "10.40.55.192:10016"; // IP address and port of the Server 2

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
   let mut server_buffer = [0; 4096];

   let mut packet_buffer = [0; 4104]; // 4096 for data + 8 for sequence number
   
   //function to send an image
   let image_path = "./src/car.png";
   let mut img = File::open(image_path)?;
   let mut buffer = Vec::new();
   // println!("Image Buffer content: {:?}", buffer);
   // iterate 400 times in a for loop
   img.read_to_end(&mut buffer)?;
   
   
   let mut image_num:u32 = 0;
   // println!("Waiting for a response from servers...");
   for i in 0..repetition_count {    
       let mut sequence_number:u64 = 1;
       
       for chunk in buffer.chunks(4096) {
            let mut packet_vector: Vec<u8> = Vec::new();
            // Clear the packet buffer
            //packet_buffer.fill(0);

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