use nix::unistd::Uid;

/// Ensures the binary is being exxecuted as root
pub fn ensure_root() {
    if !Uid::effective().is_root() {
        log::error!("This program must be run as root");
        std::process::exit(1);
    }
}
