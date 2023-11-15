use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq, Clone, Copy)]
pub enum Kind {
    Unknown = 0,
    Frontend = 1,
    Backend = 2,
    Stun = 3,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", (*self) as u8)
    }
}

impl Kind {
    pub fn from(i: u8) -> Kind {
        return match i {
            1 => Kind::Frontend,
            2 => Kind::Backend,
            3 => Kind::Stun,
            _ => Kind::Unknown,
        };
    }
}
