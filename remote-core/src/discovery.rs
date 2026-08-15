use minicbor::Encoder;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Advertisement {
    pub version: u8,
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub platform: Platform,
    pub implementation_version: String,
    pub connection_port: u16,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum DeviceType {
    Computer = 0x01,
    Phone = 0x02,
    Tablet = 0x03,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Platform {
    Linux = 0x01,
    Windows = 0x02,
    Android = 0x03,
}

impl Platform {
    pub fn from_byte(byte: u8) -> Option<Platform> {
        match byte {
            0x01 => Some(Platform::Linux),
            0x02 => Some(Platform::Windows),
            0x03 => Some(Platform::Android),
            _ => None,
        }
    }
}

impl DeviceType {
    pub fn from_byte(byte: u8) -> Option<DeviceType> {
        match byte {
            0x01 => Some(DeviceType::Computer),
            0x02 => Some(DeviceType::Phone),
            0x03 => Some(DeviceType::Tablet),
            _ => None,
        }
    }
}

impl Advertisement {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        e.map(7).unwrap();
        e.u8(1).unwrap();
        e.u8(self.version).unwrap();
        e.u8(2).unwrap();
        e.bytes(self.device_id.as_bytes()).unwrap();
        e.u8(3).unwrap();
        e.str(&self.device_name).unwrap();
        e.u8(4).unwrap();
        e.u8(self.device_type as u8).unwrap();
        e.u8(5).unwrap();
        e.u8(self.platform as u8).unwrap();
        e.u8(6).unwrap();
        e.str(&self.implementation_version).unwrap();
        e.u8(7).unwrap();
        e.u16(self.connection_port).unwrap();
        buf
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_byte() {
        assert_eq!(DeviceType::from_byte(0x01), Some(DeviceType::Computer));
        assert_eq!(DeviceType::from_byte(0x02), Some(DeviceType::Phone));
        assert_eq!(DeviceType::from_byte(0x03), Some(DeviceType::Tablet));
        assert_eq!(DeviceType::from_byte(0x04), None);
    }

    #[test]
    fn test_platform_from_byte() {
        assert_eq!(Platform::from_byte(0x01), Some(Platform::Linux));
        assert_eq!(Platform::from_byte(0x02), Some(Platform::Windows));
        assert_eq!(Platform::from_byte(0x03), Some(Platform::Android));
        assert_eq!(Platform::from_byte(0x04), None);
    }

    #[test]
    fn test_advertisement() {
        let anuncio = Advertisement {
            version: 1,
            device_id: Uuid::from_bytes([0x01; 16]),
            device_name: "Test Device".to_string(),
            device_type: DeviceType::Computer,
            platform: Platform::Windows,
            implementation_version: "1.0.0".to_string(),
            connection_port: 8080,
        };
        let bytes = anuncio.encode();

        assert_eq!(bytes[0], 0xA7);
        assert_eq!(bytes[1], 0x01);
        assert_eq!(bytes[2], 0x01);
        assert_eq!(bytes[3], 0x02);
        assert_eq!(bytes[4], 0x50);
        assert_eq!(&bytes[5..21], &[0x01; 16]);
        assert_eq!(bytes[21], 0x03);
        assert_eq!(bytes[22], 0x6B);
        assert_eq!(&bytes[23..34], b"Test Device");
        assert_eq!(bytes[34], 0x04);
        assert_eq!(bytes[35], 0x01);
        assert_eq!(bytes[36], 0x05);
        assert_eq!(bytes[37], 0x02);
        assert_eq!(bytes[38], 0x06);
        assert_eq!(bytes[39], 0x65);
        assert_eq!(&bytes[40..45], b"1.0.0");
        assert_eq!(bytes[45], 0x07);
        assert_eq!(bytes[46], 0x19);
        assert_eq!(bytes[47], 0x1F);
        assert_eq!(bytes[48], 0x90);
        assert_eq!(bytes.len(), 49);
    }
}
