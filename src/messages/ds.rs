pub mod tcp {
    use joystick::{AxisType, JoystickType};
    use states::MatchType;

    pub trait Tag {
        fn id(&self) -> u8;

        fn as_bytes(&self) -> Vec<u8>;

        fn to_packet(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.push(self.id());
            buf.extend(self.as_bytes());
            let len = buf.len();
            buf.insert(0, len as u8);

            buf
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

    pub struct MatchInfo {
        competition: String,
        match_type: MatchType,
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
