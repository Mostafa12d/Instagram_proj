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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //original
    let remote_addr1 = "10.40.60.231:10017"; // IP address and port of the Server 0
    let remote_addr2 = "10.40.60.231:10015"; // IP address and port of the Server 1
    let remote_addr3 = "10.40.60.231:10016"; // IP address and port of the Server 2
    // let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let socket1 = UdpSocket::bind("0.0.0.0:0").await?;
    let socket2 = UdpSocket::bind("0.0.0.0:0").await?;
    let socket3 = UdpSocket::bind("0.0.0.0:0").await?;
    //let mut server_buffer = [0; 4096];

   let mut server1_buffer = [0; 4096];
   let mut server2_buffer = [0; 4096];
   let mut server3_buffer = [0; 4096];


//function to send an image
    let image_path = "./src/bambo.jpeg";
    let mut img = File::open(image_path)?;
    let mut buffer = Vec::new();
    // println!("Image Buffer content: {:?}", buffer);
    // iterate 400 times in a for loop
    img.read_to_end(&mut buffer)?;
for i in 0..10 {    
    for chunk in buffer.chunks(4096) {
        //send packets to server
        println!("Sending chunk of: {}", chunk.len());
        socket1.send_to(chunk, remote_addr1).await?;
        socket2.send_to(chunk, remote_addr2).await?;
        socket3.send_to(chunk, remote_addr3).await?;
    }
    }
        println!("Sent the image to the servers");

    let mut image_num:u32 = 0;
    loop {
        println!("Waiting for a response from servers...");
        server1_buffer = [0; 4096];
        server2_buffer = [0; 4096];
        server3_buffer = [0; 4096];
        // let mut received_data = Vec::new();
        let image_string = image_num.to_string();
        let image_name = "imgs/img_rcv1".to_string() + &image_string + ".jpeg";
        let image1_string = image_num.to_string();
        let image1_name = "imgs/img_rcv2".to_string() + &image_string + ".jpeg";
        let image2_string = image_num.to_string();
        let image2_name = "imgs/img_rcv3".to_string() + &image_string + ".jpeg";
        let mut file1 = File::create(image_name)?;
        let mut file2 = File::create(image1_name)?;
        let mut file3 = File::create(image2_name)?;

        
        loop{
            // println!("Waiting for a message...");
            //receive message from client
            let (len1, server1) = socket1.recv_from(&mut server1_buffer).await?;
            let (len2, server2) = socket2.recv_from(&mut server2_buffer).await?;
            let (len3, server3) = socket3.recv_from(&mut server3_buffer).await?;

            if len1 != 0 {
                println!("Received {} bytes from {}", len1, server1);
                file1.write_all(&server1_buffer[..len1])?;
                
            }
            if len2 != 0 {
                println!("Received {} bytes from {}", len2, server2);
                file2.write_all(&server2_buffer[..len2])?;

            }   

            if len3 != 0{
                println!("Received {} bytes from {}", len3, server3);
                file3.write_all(&server3_buffer[..len3])?;
            }
            // breah after the last packet
            if len1 != 4096 || len2 != 4096 || len3 != 4096 {
                break;
            }
        }
        image_num += 1;
    }
//     loop {
//         println!("Waiting for a message...");
//         let (len, src) = socket.recv_from(&mut message_buffer).await?;
//         let message = std::str::from_utf8(&message_buffer[..len])?;
//         println!("Received: {} from {}", message, src);

//     }

    Ok(())
}

