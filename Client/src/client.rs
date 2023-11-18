use steganography::encoder::Encoder;
use steganography::util::save_image_buffer;
// This is the client
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::io::Cursor;
use std::io::Read;
use tokio::time::{sleep, Duration};
use steganography::decoder::*;
use steganography::encoder::*;
use image::DynamicImage;


// fn main(){

//     let cover = image::open("./src/bambo.jpg").unwrap();
//     let mut secret_image = Vec::new();
//     let mut secret = File::open("./src/yaboy.jpg").unwrap();
//     let mut file = File::create("./src/decoded.jpg".to_string()).unwrap();
//     secret.read_to_end(&mut secret_image).unwrap();
    
//     let encoders = Encoder::new(&secret_image, cover);

//     let encoded_image = encoders.encode_alpha();
//     save_image_buffer(encoded_image.clone(), "./src/encoded.jpg".to_string());
//    let clone = encoded_image.clone();

//     let decoded_image = Decoder::new(clone);
//     let decoded_secret = decoded_image.decode_alpha();
//     file.write_all(&decoded_secret);

    // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());

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

    //original
    let remote_addr1 = "172.29.255.134:10017"; // IP address and port of the Server 0
    let remote_addr2 = "172.29.255.134:10015"; // IP address and port of the Server 1
    let remote_addr3 = "172.29.255.134:10016"; // IP address and port of the Server 2

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
   let mut server_buffer = [0; 4096];

   let mut packet_buffer = [0; 4104]; // 4096 for data + 8 for sequence number
   
   //function to send an image
   let image_path = "./src/bambo.jpeg";
   let mut img = File::open(image_path)?;
   let mut buffer = Vec::new();
   // println!("Image Buffer content: {:?}", buffer);
   // iterate 400 times in a for loop
   img.read_to_end(&mut buffer)?;

   
   for i in 0..6 {    
        let mut sequence_number:u64 = 1;

        for chunk in buffer.chunks(4096) {
            // Clear the packet buffer
            packet_buffer.fill(0);

            // Include the sequence number in the packet
            packet_buffer[0..8].copy_from_slice(&sequence_number.to_be_bytes());
            packet_buffer[8..chunk.len()+8].copy_from_slice(chunk);

            //send packets to server
            //println!("Sending chunk of: {}", chunk.len());
            socket.send_to(&packet_buffer, remote_addr1).await?;
            socket.send_to(&packet_buffer, remote_addr2).await?;
            socket.send_to(&packet_buffer, remote_addr3).await?;

            // Increment the sequence number for the next packet
            sequence_number += 1;
        }

        //wait for NACK message
        let mut nack_buffer = [0; 4096];
        //println!("Waiting for NACK message...");
        let (len, server) = socket.recv_from(&mut nack_buffer).await?;
        println!("Received {} bytes from {}", len, server);
        
       // println!("received buffer: {:?}", nack_buffer, server);
        //if NACK message received, send the packets again to the server

        //println!("Stopped here!!!!");
        //let missing_packets = parse_nack_message(&nack_buffer[..len]);
        let s = std::str::from_utf8(&nack_buffer).expect("Invalid UTF-8");

        let mut missing_packets = Vec::new();
        for i in (0..s.len()).step_by(3) {
            // Get a slice of 3 characters, handling the case where the 
            // remaining characters are less than 3 at the end of the string
            let end = std::cmp::min(i + 3, s.len());
            missing_packets.push(&s[i..end]);
        }

        //println!("Missing packets: {:?}", missing_packets);
        if missing_packets[0] == "000"{
            println!("No missing packets");
        }
        else {
            for &seq_num in &missing_packets {
                let seq_string = seq_num.to_string();
                let seq_int = seq_string.parse::<u32>().unwrap(); // Changed to u32
        
                let packet_index = seq_int as usize * 4096;
                let end_index = std::cmp::min(packet_index + 4096, buffer.len());
                let packet_to_resend = &buffer[packet_index..end_index];
                println!("Resending packet: {} ", seq_int );
        
                packet_buffer.fill(0);
                packet_buffer[0..8].copy_from_slice(&seq_int.to_be_bytes()); // Convert seq_int to a byte array
                packet_buffer[8..8 + packet_to_resend.len()].copy_from_slice(packet_to_resend);
        
                socket.send_to(&packet_buffer, remote_addr1).await?;
                socket.send_to(&packet_buffer, remote_addr2).await?;
                socket.send_to(&packet_buffer, remote_addr3).await?;
            }
        }
            
    }
    //add 2 secs delay
    //sleep(Duration::from_secs(2)).await;

    println!("Sent the image to the servers");

    let mut image_num:u32 = 0;
    loop {
        //println!("Waiting for a response from servers...");
        // server_buffer = [0; 4096];
        // // let mut received_data = Vec::new();
        // let image_string = image_num.to_string();
        // let image_name = "imgs/img_rcv".to_string() + &image_string + ".jpeg";
        // let image_name2 = image_name.clone();
        // let mut file = File::create(image_name)?;
        // loop{
        //     // println!("Waiting for a message...");
        //     //receive message from client
        //     let (len, server) = socket.recv_from(&mut server_buffer).await?;
        //     println!("Received {} bytes from {}", len, server);
        //     file.write_all(&server_buffer[..len])?;
        //     // server_buffer = [0; 4096];
        //     // println!("Received string: {}", client);
        //     // breah after the last packet
        //     if len != 4096 {
        //         break;
        //     }
        // }
        // image_num += 1;
    }
//     loop {
//         println!("Waiting for a message...");
//         let (len, src) = socket.recv_from(&mut message_buffer).await?;
//         let message = std::str::from_utf8(&message_buffer[..len])?;
//         println!("Received: {} from {}", message, src);

//     }

    Ok(())
}