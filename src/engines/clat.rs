use ipnet::{Ipv4Net, Ipv6Net};

pub async fn do_clat(
    interface: String,
    customer_pool: Vec<Ipv4Net>,
    embed_prefix: Ipv6Net,
    num_queues: usize,
) {
}
