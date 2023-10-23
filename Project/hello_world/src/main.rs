use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target: SocketAddr = args.get(1).expect("Argument 1 is target address. Eg: 127.0.0.1:10001").parse().unwrap();

    let start_time = Instant::now();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    {
        let socket_2 = socket.try_clone().unwrap();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                // let mut user_input = String::new();
                // let stdin = io::stdin(); // We get `Stdin` here.
                // stdin.read_line(&mut user_input);

                // println!("input {} ", user_input);
                let (n, addr) = socket_2.recv_from(&mut buf).unwrap();
                if addr == target {
                    // let message = String::from_utf8_lossy(&buf[0..n]);
                    let net_elapsed_time = u64::from_le_bytes((&buf[0..n]).try_into().unwrap());
                    let elapsed_time = start_time.elapsed().as_nanos() as u64;
                    let ping = (elapsed_time as i64 - net_elapsed_time as i64) as f64 / 1e6f64;
                    println!("Received Message {}:", ping);
                }
            }
        });
    }

    loop {
        let elapsed_time = start_time.elapsed().as_nanos() as u64;
        let buf = elapsed_time.to_le_bytes();
        socket.send_to(&buf, target).unwrap();
        sleep(Duration::from_secs(1));
    }
}