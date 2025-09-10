use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::str::FromStr;

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
    pub const NAMED_COLORS: &[(&str, Self)] = &[
        ("red", Self::RED),
        ("green", Self::GREEN),
        ("blue", Self::BLUE),
    ];

    pub fn alpha(self, a: f32, base: Self) -> Self {
        Self {
            red: (self.red as f32 * a + base.red as f32 * (1.0 - a)) as u8,
            green: (self.green as f32 * a + base.green as f32 * (1.0 - a)) as u8,
            blue: (self.blue as f32 * a + base.blue as f32 * (1.0 - a)) as u8,
        }
    }

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

    pub fn from_u32(u: u32) -> Self {
        let red = u >> 16;
        let green = u >> 8;
        let blue = u;
        Self {
            red: red as u8,
            green: green as u8,
            blue: blue as u8,
        }
    }

    pub fn as_u32(self) -> u32 {
        let red = self.red as u32;
        let green = self.green as u32;
        let blue = self.blue as u32;
        (red << 16) | (green << 8) | blue
    }

    pub fn from_name(s: &str) -> Result<Self, String> {
        for named_color in Self::NAMED_COLORS {
            let (name, color) = named_color;
            if name == &s {
                return Ok(*color);
            }
        }

        Err(format!("named color \"{}\" is not found", s))
    }

    pub fn from_str_noprefix(s: &str) -> Result<Self, String> {
        match u32::from_str_radix(s, 16) {
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

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let red = (self.red as f32 * rhs) as u8;
        let green = (self.green as f32 * rhs) as u8;
        let blue = (self.blue as f32 * rhs) as u8;
        Self { red, green, blue }
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs
    }
}

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        let red = (self.red as f32 / rhs) as u8;
        let green = (self.green as f32 / rhs) as u8;
        let blue = (self.blue as f32 / rhs) as u8;
        Self { red, green, blue }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") {
            return Err("expected # character".to_string());
        }
        Self::from_str_noprefix(&s[1..])
    }
}

#[macro_export]
macro_rules! color {
    (# $x:expr) => {
        Color::from_str_noprefix(stringify!($x)).unwrap()
    };
}
