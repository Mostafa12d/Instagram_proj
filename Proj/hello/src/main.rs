use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_addr = "127.0.0.1:8080"; // IP address and port you want Node 2 to listen on

    let socket = UdpSocket::bind(local_addr).await?;
    let mut buffer = [0; 1024]; // Buffer to receive the message

    loop {
        let (len, src) = socket.recv_from(&mut buffer).await?;
        let message = std::str::from_utf8(&buffer[..len])?;

        println!("Received: {} from {}", message, src);
    }
}