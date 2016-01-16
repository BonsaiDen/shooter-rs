#[derive(Debug, Copy, Clone)]
pub enum ColorName {
    Grey,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
    Black,
    White
}

impl ColorName {

    pub fn to_u8(&self) -> u8 {
        match *self {
            ColorName::Grey => 0,
            ColorName::Red => 1,
            ColorName::Orange => 2,
            ColorName::Yellow => 3,
            ColorName::Green => 4,
            ColorName::Teal => 5,
            ColorName::Blue => 6,
            ColorName::Purple => 7,
            ColorName::Pink => 8,
            _ => 0
        }
    }

    pub fn from_u8(value: u8) -> ColorName {
        match value {
            0 => ColorName::Grey,
            1 => ColorName::Red,
            2 => ColorName::Orange,
            3 => ColorName::Yellow,
            4 => ColorName::Green,
            5 => ColorName::Teal,
            6 => ColorName::Blue,
            7 => ColorName::Purple,
            8 => ColorName::Pink,
            _ => ColorName::Grey
        }
    }

}

#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color {

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a
        }
    }

    pub fn all_colored() -> Vec<Color> {
        vec![
            Color::from_name(ColorName::Red),
            Color::from_name(ColorName::Orange),
            Color::from_name(ColorName::Yellow),
            Color::from_name(ColorName::Green),
            Color::from_name(ColorName::Teal),
            Color::from_name(ColorName::Blue),
            Color::from_name(ColorName::Purple),
            Color::from_name(ColorName::Pink)
        ]
    }

    pub fn from_name(name: ColorName) -> Color {
        match name  {
            ColorName::Grey => Color::new(0x80, 0x80, 0x80, 0xff),
            ColorName::Red => Color::new(0xf2, 0x00, 0x26, 0xff),
            ColorName::Orange => Color::new(0xfd, 0x83, 0x1c, 0xff),
            ColorName::Yellow => Color::new(0xfd, 0xda, 0x31, 0xff),
            ColorName::Green => Color::new(0x3c, 0xdc, 0x00, 0xff),
            ColorName::Teal => Color::new(0x33, 0xd0, 0xd1, 0xff),
            ColorName::Blue => Color::new(0x0f, 0x5c, 0xf9, 0xff),
            ColorName::Purple => Color::new(0x82, 0x0c, 0xe6, 0xff),
            ColorName::Pink => Color::new(0xec, 0x34, 0xa7, 0xff),
            ColorName::Black => Color::new(0x00, 0x00, 0x00, 0xff),
            ColorName::White => Color::new(0xff, 0xff, 0xff, 0xff)
        }
    }

    pub fn from_u8(value: u8) -> Color {
        Color::from_name(ColorName::from_u8(value))
    }

    pub fn from_flags(flags: u8) -> Color {
        Color::from_u8((flags & 0b1111_0000) >> 4)
    }

    pub fn to_name(&self) -> ColorName {
        match (self.r, self.g, self.b) {
            (0x80, 0x80, 0x80) => ColorName::Grey,
            (0xf2, 0x00, 0x26) => ColorName::Red,
            (0xfd, 0x83, 0x1c) => ColorName::Orange,
            (0xfd, 0xda, 0x31) => ColorName::Yellow,
            (0x3c, 0xdc, 0x00) => ColorName::Green,
            (0x33, 0xd0, 0xd1) => ColorName::Teal,
            (0x0f, 0x5c, 0xf9) => ColorName::Blue,
            (0x82, 0x0c, 0xe6) => ColorName::Purple,
            (0xec, 0x34, 0xa7) => ColorName::Pink,
            (0x00, 0x00, 0x00) => ColorName::Black,
            (0xff, 0xff, 0xff) => ColorName::White,
            (_, _, _) => ColorName::Grey
        }
    }

    pub fn to_u8(&self) -> u8 {
        Color::to_name(self).to_u8()
    }

    pub fn to_flags(&self) -> u8 {
        (self.to_u8() << 4) & 0xff
    }

    pub fn set_to(&mut self, color: &Color) {
        self.r = color.r;
        self.g = color.g;
        self.b = color.b;
        self.a = color.a;
    }

    pub fn darken(&self, by: f32) -> Color {
        let mut hsl = rgb_to_hsl(self);
        hsl.l = hsl.l * (1.0 - by);
        hsl_to_rgb(&hsl)
    }

}

#[derive(Debug)]
pub struct HSLColor {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: u8
}

pub fn rgb_to_hsl(color: &Color) -> HSLColor {

    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let mut h = 0.0;
    let mut s = 0.0;

    if max != min {

        let d = max - min;

        s = if l > 0.5 {
            d / (2.0 - max - min)

        } else {
            d / (max + min)
        };

        h = if r > g && r > b {
            (g - b) / d + if g < b {
                6.0
            } else {
                0.0
            }

        } else if g > b {
            (b - r) / d + 2.0

        } else {
            (r - g) / d + 4.0

        } / 6.0 * 360.0;

    }

    HSLColor {
        h: h,
        s: s,
        l: l,
        a: color.a
    }

}

pub fn hsl_to_rgb(color: &HSLColor) -> Color {

    let h = color.h;
    let s = color.s;
    let l = color.l;

    let m2 = if l <= 0.5 {
        l * (s + 1.0)

    } else {
        l + s - l*s
    };

    let m1 = l * 2.0 - m2;
    let h = h / 360.0;

    fn hue_to_rgb(m1: f32, m2: f32, h: f32) -> f32 {

        let h = if h < 0.0 {
            h + 1.0

        } else if h > 1.0 {
            h - 1.0

        } else {
            h
        };

        if 0.0 <= h && h < 1.0/6.0 {
            m1 + (m2 - m1)*h*6.0

        } else if 1.0/6.0 <= h && h < 1.0/2.0 {
            m2

        } else if 1.0/2.0 <= h && h < 2.0/3.0 {
            m1 + (m2 - m1)*(4.0 - 6.0*h)

        } else if 2.0/3.0 <= h && h <= 1.0 {
            m1

        } else {
            0.0
        }

    }

    Color {
        r: (255.0 * hue_to_rgb(m1, m2, h + 1.0 / 3.0)).round() as u8,
        g: (255.0 * hue_to_rgb(m1, m2, h)).round() as u8,
        b: (255.0 * hue_to_rgb(m1, m2, h - 1.0 / 3.0)).round() as u8,
        a: color.a
    }

}

