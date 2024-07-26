/// Represents a Color to print in the screen.
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn black() -> Self {
        Color::new(0, 0, 0)
    }
    pub fn white() -> Self {
        Color::new(u8::MAX, u8::MAX, u8::MAX)
    }
}

/// Converts from a hex u32 into a `Color`.
///
/// * `value`: The hex u32 to convert into a `Color`.
fn from_hex_value(value: &u32) -> Color {
    let r = ((value >> 16) & 0xFF) as u8;
    let g = ((value >> 8) & 0xFF) as u8;
    let b = (value & 0xFF) as u8;

    Color::new(r, g, b)
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        from_hex_value(&value)
    }
}
impl From<&u32> for Color {
    fn from(value: &u32) -> Self {
        from_hex_value(value)
    }
}
impl From<&mut u32> for Color {
    fn from(value: &mut u32) -> Self {
        from_hex_value(value)
    }
}

/// Converts from a color into a hex u32.
///
/// * `value`: The color to convert into a u32.
fn to_hex_value(value: &Color) -> u32 {
    let Color { r, g, b } = value;
    (*r as u32) << 16 | (*g as u32) << 8 | (*b as u32)
}

impl From<&mut Color> for u32 {
    fn from(value: &mut Color) -> Self {
        to_hex_value(value)
    }
}

impl From<&Color> for u32 {
    fn from(value: &Color) -> Self {
        to_hex_value(value)
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        to_hex_value(&value)
    }
}

impl std::ops::Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        let Color { r, g, b } = self;
        let Color {
            r: r2,
            g: g2,
            b: b2,
        } = rhs;

        Color::new(
            r.saturating_add(r2),
            g.saturating_add(g2),
            b.saturating_add(b2),
        )
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, factor: f32) -> Self::Output {
        let Color { r, g, b } = self;

        Color::new(
            (r as f32 * factor).clamp(0.0, 255.0) as u8,
            (g as f32 * factor).clamp(0.0, 255.0) as u8,
            (b as f32 * factor).clamp(0.0, 255.0) as u8,
        )
    }
}

impl std::ops::Sub for Color {
    type Output = Color;

    fn sub(self, rhs: Self) -> Self::Output {
        let Color { r, g, b } = self;
        let Color {
            r: r2,
            g: g2,
            b: b2,
        } = rhs;

        Color::new(
            r.saturating_sub(r2),
            g.saturating_sub(g2),
            b.saturating_sub(b2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_by_negative() {
        let color = Color::new(5, 100, 1);
        let factor = -1.5;

        let Color { r, g, b } = color * factor;

        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_multiply_by_large_number() {
        let color = Color::new(255, 100, 1);
        let factor = 100000.0;

        let Color { r, g, b } = color * factor;

        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }
}
