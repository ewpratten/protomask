use crate::types::IpPacket;

pub fn get_protocol<'a>(packet: IpPacket<'a>) -> u8 {
    packet[0] >> 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_version_ipv4(){
        let packet = [0x40u8, 0x00u8, 0x00u8, 0x00u8];
        assert_eq!(get_protocol(&packet), 4);
    }
}