pub mod tcp {
    use crate::joystick::{AxisType, JoystickType};
    use crate::states::MatchType;

	use byteorder::{WriteBytesExt, NetworkEndian};

    pub trait Tag {
        fn id(&self) -> u8;

        fn as_bytes(&self) -> Vec<u8>;

        fn to_packet(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.push(self.id());
            buf.extend(self.as_bytes());
			
            let len = buf.len();
			let mut packet = Vec::new();
			packet.write_u16::<NetworkEndian>(len as u16);
            packet.extend(buf);

            packet
        }
    }

    pub enum TcpTag {
        JoystickDescriptor(JoystickDescriptor),
        MatchInfo(MatchInfo),
        GameData(GameData),
    }

    impl TcpTag {
        pub fn to_packet(&self) -> Vec<u8> {
            match self {
                TcpTag::JoystickDescriptor(jd) => Vec::new(), // TODO
                TcpTag::MatchInfo(mi) => mi.to_packet(),
                TcpTag::GameData(gd) => gd.to_packet(),
            }
        }
    }

    pub struct JoystickDescriptor {
        index: u8,
        is_xbox: bool,
        stick_type: JoystickType,
        name: String,
        axis_count: u8,
        axis_types: Vec<AxisType>,
        button_count: u8,
        pov_count: u8,
    }

    #[derive(Clone)]
    pub struct MatchInfo {
        pub competition: String,
        pub match_type: MatchType,
    }

    impl Tag for MatchInfo {
        fn id(&self) -> u8 {
            0x07
        }

        fn as_bytes(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.extend(self.competition.as_bytes());
            buf.push(self.match_type as u8);

            buf
        }
    }

    pub struct GameData {
        data: String,
    }

    impl GameData {
        pub fn new(data: String) -> Self {
            Self { data }
        }
    }

    impl Tag for GameData {
        fn id(&self) -> u8 {
            0x0e
        }

        fn as_bytes(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.extend(self.data.as_bytes());

            buf
        }
    }
}
