// This is the client
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::io::Cursor;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage};
use base64::{Engine as _, engine::general_purpose};
use base64::encode;
use base64::decode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //Specify address form file
    // let filename = "DoSC.txt";
    // let file = File::open(filename).expect("no such file");
    // let buf = BufReader::new(file);
    // let local_addr_v: Vec<String> = buf.lines()
    // .map(|l| l.expect("Could not parse line"))
    // .collect();
    // for addr in &local_addr_v{
    //     println!("{}", addr);
    // }

    //Specify address from command line
    // let args: Vec<String> = std::env::args().collect();
    // let local_addr = args.get(1).expect("Argument 1 is listening address. Eg: 0.0.0.0:10001");

    // //Specify address from code
    //let local_addr = "10.40.32.144:10011"; 

    //original
    let remote_addr1 = "172.20.10.4:10018"; // IP address and port of the Server 1
    let remote_addr2 = "172.20.10.4:10019"; // IP address and port of the Server 2
    let remote_addr3 = "172.20.10.4:10020"; // IP address and port of the Server 3

    
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let message1 = "Hello, Server18!";
    let message2 = "Hello, Server19!";
    let message3 = "Hello, Server20!";

    let message_bytes1 = message1.as_bytes();  
    let message_bytes2 = message2.as_bytes();  
    let message_bytes3 = message3.as_bytes();  


    socket.send_to(message_bytes1, remote_addr1).await?; // Send the message

    socket.send_to(message_bytes2, remote_addr2).await?;
    socket.send_to(message_bytes3, remote_addr3).await?;
    // socket.send_to(message_bytes, remote_addr1).await?;

    // println!("Sent: {} to {}", message, remote_addr1);
    
    println!("Sent: {} to {}", message1, remote_addr1);
    println!("Sent: {} to {}", message2, remote_addr2);
    println!("Sent: {} to {}", message3, remote_addr3);

    //Receive Reply from server
    let mut buffer = [0; 1024]; // Buffer to receive the message

    let image_path = "./src/tamer.jpeg";
    let img = image::open(image_path).unwrap();

    // Convert the image to a base64-encoded string
    let base64_image_str = image_to_base64_string(img);
    let chunk_size = 1000;
    let mut offset = 0;

    while offset < base64_image_str.len() {
        let end = offset + chunk_size;
        let end = if end > base64_image_str.len() {
            base64_image_str.len()
        } else {
            end
        };
    
        let data_chunk = &base64_image_str[offset..end].as_bytes();
        offset = end;

        // Send each chunk to the client
        socket.send_to(data_chunk, remote_addr1).await?;
        println!("Sent: {} to {}", end, remote_addr1);
    }


    //decode
    let base64_jpeg_image_str = base64_image_str;
    // Decode the base64-encoded JPEG string back into a DynamicImage
    let decoded_image = decode_base64_jpeg_string(&base64_jpeg_image_str);

    decoded_image.save("ouut.jpeg").unwrap();

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);
    }

    Ok(())
}

fn image_to_base64_string(img: DynamicImage) -> String {
    // Ensure that the image is in the RGBA8 format
    let img = img.to_rgba8();

    // Create a Vec<u8> to store the image data
    let mut buffer = Vec::new();

    let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut buffer);
    encoder.encode(&img, img.width(), img.height(), image::ColorType::Rgba8)
        .expect("Failed to encode image as JPEG");

    // Encode the binary data as a base64 string
    let base64_str = encode(&buffer);

    base64_str
}

fn decode_base64_jpeg_string(base64_jpeg_str: &str) -> DynamicImage {
    // Decode the base64 string to obtain binary image data
    let decoded_data = decode(base64_jpeg_str).expect("Failed to decode base64 string");

    // Create a Reader from the binary image data
    let reader = std::io::Cursor::new(decoded_data);

    // Decode the image using the image crate's `decode` function
    image::load(reader, image::ImageFormat::Jpeg)
        .expect("Failed to decode JPEG image")
}