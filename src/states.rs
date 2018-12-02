#[derive(Copy, Clone)]
pub enum RobotMode {
    Teleop = 0,
    Test = 1,
    Auto = 2,
}

impl RobotMode {
    pub fn from(val: u8) -> Option<Self> {
        match val {
            0 => Some(RobotMode::Teleop),
            1 => Some(RobotMode::Test),
            2 => Some(RobotMode::Auto),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum Alliance {
    Red(u8),
    Blue(u8),
}

impl Alliance {
    // TODO check bounds on both methods

    pub fn to_position_u8(&self) -> u8 {
        match self {
            Alliance::Red(pos) => pos - 1,
            Alliance::Blue(pos) => pos + 2,
        }
    }

    pub fn from_position_u8(pos: u8) -> Self {
        if pos < 3 {
            Alliance::Red(pos + 1)
        } else {
            Alliance::Blue(pos % 3 + 1)
        }
    }
}

#[derive(Copy, Clone)]
pub enum MatchType {
    None = 0,
    Practice = 1,
    Qualification = 2,
    Elimination = 3,
}
