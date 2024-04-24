use std::io::Read;

use easy_tun::Tun;

fn main() {
    // Enable logs
    env_logger::init();

    // Bring up a TUN interface
    let tun = Tun::new("tun%d", 1).unwrap();

    // Loop and read from the interface
    let mut buffer = [0u8; 1500];
    loop {
        let length = tun.fd(0).unwrap().read(&mut buffer).unwrap();
        println!("{:?}", &buffer[..length]);
    }
}
