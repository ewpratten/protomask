//! Entrypoint for the `protomask-clat` binary.
//!
//! This binary is a Customer-side transLATor (CLAT) that translates all native
//! IPv4 traffic to IPv6 traffic for transmission over an IPv6-only ISP network.

use clap::Parser;
use common::logging::enable_logger;
use easy_tun::Tun;
use ipnet::Ipv6Net;
use nix::unistd::Uid;

mod common;

#[derive(Debug, Parser)]
#[clap(author, version, about="IPv4 to IPv6 Customer-side transLATor (CLAT)", long_about = None)]
struct Args {
    /// IPv6 prefix to embed IPv4 addresses in
    #[clap(long="via", default_value_t = ("64:ff9b::/96").parse().unwrap())]
    embed_prefix: Ipv6Net,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("clat%d").to_string())]
    interface: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = Args::parse();

    // Initialize logging
    enable_logger(args.verbose);

    // We must be root to continue program execution
    if !Uid::effective().is_root() {
        log::error!("This program must be run as root");
        std::process::exit(1);
    }

    // Bring up a TUN interface
    let mut tun = Tun::new(&args.interface).unwrap();

    log::info!("Translating packets on {}", tun.name());
    loop {

    }
}
