extern crate allegro;

#[derive(Copy, Clone)]
pub struct Color((u8, u8, u8));

impl Color {

    pub fn black() -> Color {
        Color((0x00, 0x00, 0x00))
    }

    pub fn red() -> Color {
        Color((0xf2, 0x00, 0x26))
    }

    pub fn orange() -> Color {
        Color((0xfd, 0x83, 0x1c))
    }

    pub fn yellow() -> Color {
        Color((0xfd, 0xda, 0x31))
    }

    pub fn green() -> Color {
        Color((0x3c, 0xdc, 0x00))
    }

    pub fn teal() -> Color {
        Color((0x33, 0xd0, 0xd1))
    }

    pub fn blue() -> Color {
        Color((0x0f, 0x5c, 0xf9))
    }

    pub fn purple() -> Color {
        Color((0x82, 0x0c, 0xe6))
    }

    pub fn pink() -> Color {
        Color((0xec, 0x34, 0xa7))
    }

    pub fn map_rgb(&self, core: &allegro::Core) -> allegro::Color {
        let r: u8 = (self.0).0;
        let g: u8 = (self.0).1;
        let b: u8 = (self.0).2;
        core.map_rgb(r, g, b)
    }

    pub fn map_rgba(&self, core: &allegro::Core, alpha: f32) -> allegro::Color {
        core.map_rgb(
            ((self.0).0 as f32 * alpha) as u8,
            ((self.0).1 as f32 * alpha) as u8,
            ((self.0).2 as f32 * alpha) as u8
        )
    }

    pub fn darken(&self, by: f32) -> Color {
        let (h, s, l) = rgb_to_hsl(self.0);
        Color(hsl_to_rgb((h, s, l * (1.0 - by))))
    }

}

pub fn rgb_to_hsl(color: (u8, u8, u8)) -> (f32, f32, f32) {

    let r = color.0 as f32 / 255.0;
    let g = color.1 as f32 / 255.0;
    let b = color.2 as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;

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

    (h, s, l)

}

pub fn hsl_to_rgb(color: (f32, f32, f32)) -> (u8, u8, u8) {

    let (h, s, l) = color;

    let m2 = if l <= 0.5 { l*(s + 1.0) } else { l + s - l*s };
    let m1 = l*2.0 - m2;
    let h = h / 360.0;

    fn hue_to_rgb(m1: f32, m2: f32, h: f32) -> f32 {
        let h = if h < 0.0 { h + 1.0 } else if h > 1.0 { h - 1.0 } else { h };

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

    let r = (255.0 * hue_to_rgb(m1, m2, h + 1.0 / 3.0)).round() as u8;
    let g = (255.0 * hue_to_rgb(m1, m2, h)).round() as u8;
    let b = (255.0 * hue_to_rgb(m1, m2, h - 1.0 / 3.0)).round() as u8;

    (r, g, b)

}

