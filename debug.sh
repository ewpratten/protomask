#! /bin/bash
# A little script to isolate and run protomask for testing
set -ex

# Set up network namespace
ip netns del protomask || true
ip netns add protomask
ip netns exec protomask ip link set lo up
ip netns exec protomask ip link add test1 type dummy
ip netns exec protomask ip link set test1 up
ip netns exec protomask ip addr add 2001:db8:1::2 dev test1
ip netns exec protomask ip link add test2 type dummy
ip netns exec protomask ip link set test2 up
ip netns exec protomask ip addr add 172.16.10.2 dev test2

# Turn off the firewall for the test interfaces
ip netns exec protomask firewall-cmd --zone=trusted --add-interface=nat64i0
ip netns exec protomask firewall-cmd --zone=trusted --add-interface=test1
ip netns exec protomask firewall-cmd --zone=trusted --add-interface=test2

# Run protomask
ip netns exec protomask ./target/debug/protomask protomask.toml -v


