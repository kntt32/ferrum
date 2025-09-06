use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {}

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
