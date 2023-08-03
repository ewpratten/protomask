use clap::Parser;
use common::{logging::enable_logger, rfc6052::parse_network_specific_prefix};
use easy_tun::Tun;
use fast_nat::CrossProtocolNetworkAddressTable;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use nix::unistd::Uid;
use std::{
    io::{BufRead, Read, Write},
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

mod common;

#[derive(Parser)]
#[clap(author, version, about="Fast and simple NAT64", long_about = None)]
struct Args {
    /// RFC6052 IPv6 translation prefix
    #[clap(long, default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    translation_prefix: Ipv6Net,

    #[command(flatten)]
    pool: PoolArgs,

    /// A CSV file containing static address mappings from IPv6 to IPv4
    #[clap(long = "static-file")]
    static_file: Option<PathBuf>,

    /// NAT reservation timeout in seconds
    #[clap(long, default_value = "7200")]
    reservation_timeout: u64,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("nat%d").to_string())]
    interface: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

impl Args {
    pub fn get_static_reservations(
        &self,
    ) -> Result<Vec<(Ipv6Addr, Ipv4Addr)>, Box<dyn std::error::Error>> {
        log::warn!("Static reservations are not yet implemented");
        Ok(Vec::new())
    }
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct PoolArgs {
    /// IPv4 prefixes to use as NAT pool address space
    #[clap(long = "pool-add")]
    pool_prefixes: Vec<Ipv4Net>,

    /// A file containing newline-delimited IPv4 prefixes to use as NAT pool address space
    #[clap(long = "pool-file", conflicts_with = "pool_prefixes")]
    pool_file: Option<PathBuf>,
}

impl PoolArgs {
    /// Read all pool prefixes from the chosen source
    pub fn prefixes(&self) -> Result<Vec<Ipv4Net>, Box<dyn std::error::Error>> {
        match self.pool_prefixes.len() > 0 {
            true => Ok(self.pool_prefixes.clone()),
            false => {
                let mut prefixes = Vec::new();
                let file = std::fs::File::open(self.pool_file.as_ref().unwrap())?;
                let reader = std::io::BufReader::new(file);
                for line in reader.lines() {
                    let line = line?;
                    let prefix = line.parse::<Ipv4Net>()?;
                    prefixes.push(prefix);
                }
                Ok(prefixes)
            }
        }
    }
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
    log::debug!("Creating new TUN interface");
    let mut tun = Tun::new(&args.interface).unwrap();
    log::debug!("Created TUN interface: {}", tun.name());

    // Get the interface index
    let rt_handle = rtnl::new_handle().unwrap();
    let tun_link_idx = rtnl::link::get_link_index(&rt_handle, tun.name())
        .await
        .unwrap()
        .unwrap();

    // Bring the interface up
    rtnl::link::link_up(&rt_handle, tun_link_idx).await.unwrap();

    // Add a route for the translation prefix
    log::debug!(
        "Adding route for {} to {}",
        args.translation_prefix,
        tun.name()
    );
    rtnl::route::route_add(IpNet::V6(args.translation_prefix), &rt_handle, tun_link_idx)
        .await
        .unwrap();

    // Add a route for each NAT pool prefix
    for pool_prefix in args.pool.prefixes().unwrap() {
        log::debug!("Adding route for {} to {}", pool_prefix, tun.name());
        rtnl::route::route_add(IpNet::V4(pool_prefix), &rt_handle, tun_link_idx)
            .await
            .unwrap();
    }

    // Set up the address table
    let mut addr_table = CrossProtocolNetworkAddressTable::default();
    for (v6_addr, v4_addr) in args.get_static_reservations().unwrap() {
        addr_table.insert_indefinite(v4_addr, v6_addr);
    }

    // Translate all incoming packets
    log::info!("Translating packets on {}", tun.name());
    let mut buffer = vec![0u8; 1500];
    // loop {
    //     // Read a packet
    //     let len = tun.read(&mut buffer).unwrap();

    //     // Translate it based on the Layer 3 protocol number
    //     if let Some(output) = handle_packet(
    //         &buffer[..len],
    //         // IPv4 -> IPv6
    //         |packet, source, dest| {
    //             // translate_ipv4_to_ipv6(
    //             //     packet,
    //             //     unsafe { embed_ipv4_addr_unchecked(*source, args.embed_prefix) },
    //             //     unsafe { embed_ipv4_addr_unchecked(*dest, args.embed_prefix) },
    //             // )
    //             todo!()
    //         },
    //         // IPv6 -> IPv4
    //         |packet, source, dest| {

    //             // translate_ipv6_to_ipv4(
    //             //     packet,
    //             //     unsafe { extract_ipv4_addr_unchecked(*source, args.embed_prefix.prefix_len()) },
    //             //     unsafe { extract_ipv4_addr_unchecked(*dest, args.embed_prefix.prefix_len()) },
    //             // )
    //             todo!()
    //         },
    //     ) {
    //         // Write the packet if we get one back from the handler functions
    //         tun.write_all(&output).unwrap();
    //     }
    // }
}
