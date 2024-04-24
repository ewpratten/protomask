use std::{io::Read, sync::Arc};

use easy_tun::Tun;

fn main() {
    // Enable logs
    env_logger::init();

    // Bring up a TUN interface
    let tun = Arc::new(Tun::new("tun%d", 5).unwrap());

    // Spawn 5 threads to read from the interface
    let mut threads = Vec::new();
    for i in 0..5 {
        let tun = Arc::clone(&tun);
        threads.push(std::thread::spawn(move || {
            let mut buffer = [0u8; 1500];
            loop {
                let length = tun.fd(i).unwrap().read(&mut buffer).unwrap();
                println!("Queue #{}: {:?}", i, &buffer[..length]);
            }
        }));
    }

    // Wait for all threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
}
