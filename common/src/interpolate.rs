/// Linear interpolation
pub fn interpolate_1_d(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

// Bilinear interpolation
pub fn interpolate_2_d(_00: f32, _10: f32, _01: f32, _11: f32, t: f32, u: f32) -> f32 {
    interpolate_1_d(
        interpolate_1_d(_00, _10, t),
        interpolate_1_d(_01, _11, t),
        u,
    )
}

// Trilinear interpolation
pub fn interpolate_3_d(
    _000: f32,
    _100: f32,
    _010: f32,
    _110: f32,
    _001: f32,
    _101: f32,
    _011: f32,
    _111: f32,
    t: f32,
    u: f32,
    v: f32,
) -> f32 {
    interpolate_1_d(
        interpolate_2_d(_000, _001, _010, _011, u, v),
        interpolate_2_d(_100, _101, _110, _111, u, v),
        t,
    )
}

#[cfg(test)]
mod interpolation {

    use crate::interpolate::*;

    #[test]
    fn interpolation() {
        assert_eq!(0.0, interpolate_1_d(0.0, 1.0, 0.0));
        assert_eq!(1.0, interpolate_1_d(0.0, 1.0, 1.0));
        assert_eq!(0.5, interpolate_1_d(0.0, 1.0, 0.5));
    }
}
