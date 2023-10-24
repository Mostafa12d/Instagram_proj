// This is the server
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_addr = "127.0.0.1:8080"; // IP address and port you want the server (this process) to listen on
    let socket = UdpSocket::bind(local_addr).await?;
    let mut buffer = vec![0; 4096]; // Buffer to receive the message
    let (num_bytes, _) = socket.recv_from(&mut buf)?;
    let image_data = &buf[..num_bytes];
    
    let decoder = PngDecoder::new(std::io::Cursor::new(image_data));
    let (width, height) = decoder.dimensions();
    let image = decoder.read_image()?;

    // Save or display the reconstructed image
    image.save("reconstructed_image.png")?;

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);
    }
}