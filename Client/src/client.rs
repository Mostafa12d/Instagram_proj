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


fn main(){

    let cover = image::open("./src/bambo.jpg").unwrap();
    let mut secret_image = Vec::new();
    let mut secret = File::open("./src/yaboy.jpg").unwrap();
    let mut file = File::create("./src/decoded.jpg".to_string()).unwrap();
    secret.read_to_end(&mut secret_image).unwrap();
    
    let encoders = Encoder::new(&secret_image, cover);

    let encoded_image = encoders.encode_alpha();
    save_image_buffer(encoded_image.clone(), "./src/encoded.jpg".to_string());
   let clone = encoded_image.clone();

    let decoded_image = Decoder::new(clone);
    let decoded_secret = decoded_image.decode_alpha();
    file.write_all(&decoded_secret);

    // save_image_buffer(decoded_secret, "./src/decoded.jpg".to_string());

}


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {

//     //original
//     let remote_addr1 = "10.40.38.59:10014"; // IP address and port of the Server 0
//     let remote_addr2 = "10.40.38.59:10019"; // IP address and port of the Server 1
//     let remote_addr3 = "10.40.38.59:10020"; // IP address and port of the Server 2

//     let socket = UdpSocket::bind("0.0.0.0:0").await?;
//    let mut message_buffer = [0; 5000];

// //function to send an image
//     let image_path = "./src/bambo.jpeg";
//     let mut img = File::open(image_path)?;
//     let mut buffer = Vec::new();
//     // println!("Image Buffer content: {:?}", buffer);
//     // iterate 400 times in a for loop
//     img.read_to_end(&mut buffer)?;
//     for i in 0..100 {    
//         for chunk in buffer.chunks(4096) {
//             //send packets to server
//             println!("Sending chunk of: {}", chunk.len());
//             socket.send_to(chunk, remote_addr1).await?;
//             socket.send_to(chunk, remote_addr2).await?;
//             socket.send_to(chunk, remote_addr3).await?;
//         }
//     }

//     loop {
//         println!("Waiting for a message...");
//         let (len, src) = socket.recv_from(&mut message_buffer).await?;
//         let message = std::str::from_utf8(&message_buffer[..len])?;
//         println!("Received: {} from {}", message, src);

//     }

//     Ok(())
// }

