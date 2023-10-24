// This is the server
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_addr = "0.0.0.0:8082"; // IP address and port you want the server (this process) to listen on
    let socket = UdpSocket::bind(local_addr).await?;
    let mut buffer = [0; 1024]; // Buffer to receive the message

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);

        // Send a response back to the client
        let response = "Hello, client!"; // Change this to your response message
        let sent_len = socket.send_to(response.as_bytes(), &src).await?;
        println!("Sent: {} bytes to {}", sent_len, src);
    }
}