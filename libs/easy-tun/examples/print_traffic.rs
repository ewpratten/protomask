use easy_tun::Tun;
use std::io::Read;

fn main() {
    // Enable logs
    env_logger::init();

    // Bring up a TUN interface
    let mut tun = Tun::new("tun%d").unwrap();

    // Loop and read from the interface
    let mut buffer = [0u8; 1500];
    loop {
        let length = tun.read(&mut buffer).unwrap();
        println!("{:?}", &buffer[..length]);
    }
}
