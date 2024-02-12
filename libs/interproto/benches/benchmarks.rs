use criterion::{criterion_group, criterion_main, Criterion};
use interproto::protocols::*;
use pnet::packet::{
    tcp::{MutableTcpPacket, TcpPacket},
    udp::{MutableUdpPacket, UdpPacket},
};

/// Translate TCP packets from IPv4 to IPv6
fn bench_tcp_4_to_6(c: &mut Criterion) {
    // Create a test input packet
    let mut input_buffer = vec![0u8; TcpPacket::minimum_packet_size() + 13];
    let mut input_packet = MutableTcpPacket::new(&mut input_buffer).unwrap();
    input_packet.set_source(1234);
    input_packet.set_destination(5678);
    input_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

    // Pre-calculate the source and dest addrs
    let source = "2001:db8::1".parse().unwrap();
    let dest = "2001:db8::2".parse().unwrap();

    // Build a benchmark group for measuring throughput
    let mut group = c.benchmark_group("tcp_4_to_6");
    group.throughput(criterion::Throughput::Bytes(input_buffer.len() as u64));
    group.bench_function("translate", |b| {
        b.iter(|| tcp::recalculate_tcp_checksum_ipv6(&input_buffer, source, dest))
    });
    group.finish();
}

/// Translate TCP packets from IPv6 to IPv4
fn bench_tcp_6_to_4(c: &mut Criterion) {
    // Create a test input packet
    let mut input_buffer = vec![0u8; TcpPacket::minimum_packet_size() + 13];
    let mut input_packet = MutableTcpPacket::new(&mut input_buffer).unwrap();
    input_packet.set_source(1234);
    input_packet.set_destination(5678);
    input_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

    // Pre-calculate the source and dest addrs
    let source = "192.0.2.1".parse().unwrap();
    let dest = "192.0.2.2".parse().unwrap();

    // Build a benchmark group for measuring throughput
    let mut group = c.benchmark_group("tcp_6_to_4");
    group.throughput(criterion::Throughput::Bytes(input_buffer.len() as u64));
    group.bench_function("translate", |b| {
        b.iter(|| tcp::recalculate_tcp_checksum_ipv4(&input_buffer, source, dest))
    });
    group.finish();
}

/// Translate UDP packets from IPv4 to IPv6
fn bench_udp_4_to_6(c: &mut Criterion) {
    // Create a test input packet
    let mut input_buffer = vec![0u8; UdpPacket::minimum_packet_size() + 13];
    let mut udp_packet = MutableUdpPacket::new(&mut input_buffer).unwrap();
    udp_packet.set_source(1234);
    udp_packet.set_destination(5678);
    udp_packet.set_length(13);
    udp_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

    // Pre-calculate the source and dest addrs
    let source = "2001:db8::1".parse().unwrap();
    let dest = "2001:db8::2".parse().unwrap();

    // Build a benchmark group for measuring throughput
    let mut group = c.benchmark_group("udp_4_to_6");
    group.throughput(criterion::Throughput::Bytes(input_buffer.len() as u64));
    group.bench_function("translate", |b| {
        b.iter(|| udp::recalculate_udp_checksum_ipv6(&input_buffer, source, dest))
    });
    group.finish();
}

/// Translate UDP packets from IPv6 to IPv4
fn bench_udp_6_to_4(c: &mut Criterion) {
    // Create a test input packet
    let mut input_buffer = vec![0u8; UdpPacket::minimum_packet_size() + 13];
    let mut udp_packet = MutableUdpPacket::new(&mut input_buffer).unwrap();
    udp_packet.set_source(1234);
    udp_packet.set_destination(5678);
    udp_packet.set_length(13);
    udp_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

    // Pre-calculate the source and dest addrs
    let source = "192.0.2.1".parse().unwrap();
    let dest = "192.0.2.2".parse().unwrap();

    // Build a benchmark group for measuring throughput
    let mut group = c.benchmark_group("udp_6_to_4");
    group.throughput(criterion::Throughput::Bytes(input_buffer.len() as u64));
    group.bench_function("translate", |b| {
        b.iter(|| udp::recalculate_udp_checksum_ipv4(&input_buffer, source, dest))
    });
    group.finish();
}

// Generate a main function
criterion_group!(
    benches,
    bench_tcp_4_to_6,
    bench_tcp_6_to_4,
    bench_udp_4_to_6,
    bench_udp_6_to_4
);
criterion_main!(benches);
