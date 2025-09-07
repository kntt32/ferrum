use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::str::FromStr;
use std::ops::AddAssign;
use std::ops::Add;
use std::ops::SubAssign;
use std::ops::Sub;
use std::ops::MulAssign;
use std::ops::Mul;
use std::ops::DivAssign;
use std::ops::Div;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        red: 0xff,
        green: 0xff,
        blue: 0xff,
    };
    pub const BLACK: Self = Self {
        red: 0x00,
        green: 0x00,
        blue: 0x00,
    };
    pub const RED: Self = Self {
        red: 0xff,
        green: 0x00,
        blue: 0x00,
    };
    pub const GREEN: Self = Self {
        red: 0x00,
        green: 0xff,
        blue: 0x00,
    };
    pub const BLUE: Self = Self {
        red: 0x00,
        green: 0x00,
        blue: 0xff,
    };

    pub fn rotate(self) -> Self {
        let Self {
            red: r,
            green: g,
            blue: b,
        } = self;
        Self {
            red: b,
            green: r,
            blue: g,
        }
    }

    pub fn as_u32(self) -> u32 {
        let red = self.red as u32;
        let green = self.green as u32;
        let blue = self.blue as u32;
        (red << 16) | (green << 8) | blue
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#{:x}{:x}{:x}", self.red, self.green, self.blue)
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") {
            return Err("expected # character".to_string());
        }
        match u32::from_str_radix(&s[1..], 16) {
            Ok(value) => {
                let red = (value >> 16) as u8;
                let green = (value >> 8) as u8;
                let blue = value as u8;
                Ok(Self { red, green, blue })
            }
            Err(e) => Err(format!("{}", e)),
        }
    }
}

#[macro_export]
macro_rules! color {
    ($x:expr) => {
        $x.parse::<Color>().unwrap()
    };
}
