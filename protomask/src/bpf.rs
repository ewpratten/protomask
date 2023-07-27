use aya::include_bytes_aligned;
use aya::Bpf;
use aya::BpfError;
use cfg_if::cfg_if;

#[cfg(all(target_os = "linux", target_arch = "x86_64", debug_assertions))]
pub fn load_bpf() -> Result<Bpf, BpfError> {
    Bpf::load(&include_bytes_aligned!("../../target/bpfel-unknown-none/debug/protomask-ebpf"))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", not(debug_assertions)))]
pub fn load_bpf() -> Result<Bpf, BpfError> {
    Bpf::load(&include_bytes_aligned!("../../target/bpfel-unknown-none/release/protomask-ebpf"))
}

