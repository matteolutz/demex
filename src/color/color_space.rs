use strum::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EnumIter)]
pub enum RgbColorSpace {
    #[default]
    Srgb,

    AdobeRgb,
    WideGamutRgb,
}

impl std::fmt::Display for RgbColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Srgb => write!(f, "sRGB"),
            Self::AdobeRgb => write!(f, "Adobe RGB"),
            Self::WideGamutRgb => write!(f, "Wide Gamut RGB"),
        }
    }
}

impl RgbColorSpace {
    pub fn rgb_to_xyz(self) -> nalgebra::SMatrix<f32, 3, 3> {
        match self {
            Self::Srgb => nalgebra::matrix![
                0.412_456_4,  0.357_576_1,  0.180_437_5;
                0.212_672_9,  0.715_152_2,  0.072_175_0;
                0.019_333_9,  0.119_192_0,  0.950_304_1
            ],
            Self::AdobeRgb => nalgebra::matrix![
                0.576_730_9,  0.185_554_0,  0.188_185_2;
                0.297_376_9,  0.627_349_1,  0.075_274_1;
                0.027_034_3,  0.070_687_2,  0.991_108_5
            ],
            Self::WideGamutRgb => nalgebra::matrix![
                0.716_104_6,  0.100_929_6,  0.147_185_8;
                0.258_187_4,  0.724_937_8,  0.016_874_8;
                0.000_000_0,  0.051_781_3,  0.773_428_7
            ],
        }
    }

    pub fn xyz_to_rgb(self) -> nalgebra::SMatrix<f32, 3, 3> {
        match self {
            Self::Srgb => nalgebra::matrix![
                3.240_454_2, -1.537_138_5, -0.498_531_4;
                -0.969_266_0, 1.876_010_8,  0.041_556_0;
                0.055_643_4, -0.204_025_9,  1.057_225_2
            ],
            Self::AdobeRgb => nalgebra::matrix![
                2.041_369_0, -0.564_946_4, -0.344_694_4;
                -0.969_266_0,  1.876_010_8,  0.041_556_0;
                0.013_447_4, -0.118_389_7,  1.015_409_6;
            ],
            Self::WideGamutRgb => nalgebra::matrix![
                1.462_806_7, -0.184_062_3, -0.274_360_6;
               -0.521_793_3,  1.447_238_1,  0.067_722_7;
               0.034_934_2, -0.096_893_0,  1.288_409_9
            ],
        }
    }

    pub fn gammut(self) -> [RgbValue; 3] {
        [
            RgbValue::new(1.0, 0.0, 0.0, self), // Red
            RgbValue::new(0.0, 1.0, 0.0, self), // Green
            RgbValue::new(0.0, 0.0, 1.0, self), // Blue
        ]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RgbValue {
    r: f32,
    g: f32,
    b: f32,
    color_space: RgbColorSpace,
}

impl RgbValue {
    pub fn new(r: f32, g: f32, b: f32, color_space: RgbColorSpace) -> Self {
        Self {
            r,
            g,
            b,
            color_space,
        }
    }

    pub fn from_color(color: egui::Color32, color_space: RgbColorSpace) -> Self {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;
        Self::new(r, g, b, color_space)
    }

    pub fn invert(mut self) -> Self {
        self.r = 1.0 - self.r;
        self.g = 1.0 - self.g;
        self.b = 1.0 - self.b;
        self
    }

    pub fn to_xy(&self) -> (f32, f32) {
        let rgb_matrix = self.color_space.rgb_to_xyz();
        let rgb_vector = nalgebra::vector![self.r, self.g, self.b];
        let xyz_vector = rgb_matrix * rgb_vector;
        let (x, y, z) = (xyz_vector[0], xyz_vector[1], xyz_vector[2]);

        let sum = x + y + z;
        (x / sum, y / sum)
    }

    pub fn to_bri(&self) -> f32 {
        let rgb_matrix = self.color_space.rgb_to_xyz();
        let rgb_vector = nalgebra::vector![self.r, self.g, self.b];
        let xyz_vector = rgb_matrix * rgb_vector;
        xyz_vector[1] // Return the Y component as brightness
    }

    pub fn from_xyz(x: f32, y: f32, z: f32, color_space: RgbColorSpace) -> Self {
        let xyz_vector = nalgebra::vector![x, y, z];
        let rgb_matrix = color_space.xyz_to_rgb();
        let mut rgb_vector = rgb_matrix * xyz_vector;

        let max = rgb_vector.max();
        if max > 1.0 {
            let scale = 1.0 / max;
            rgb_vector = rgb_vector.scale(scale);
        }

        Self::new(
            rgb_vector[0].clamp(0.0, 1.0),
            rgb_vector[1].clamp(0.0, 1.0),
            rgb_vector[2].clamp(0.0, 1.0),
            color_space,
        )
    }

    pub fn from_xyy(x: f32, y: f32, big_y: f32, color_space: RgbColorSpace) -> Self {
        if y == 0.0 {
            return Self::new(0.0, 0.0, 0.0, color_space);
        }

        let big_x = (x * big_y) / y;
        let big_z = ((1.0 - x - y) * big_y) / y;

        Self::from_xyz(big_x, big_y, big_z, color_space)
    }
}

impl From<RgbValue> for ecolor::Color32 {
    fn from(value: RgbValue) -> Self {
        let (r, g, b) = (value.r, value.g, value.b);
        ecolor::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

impl From<RgbValue> for [f32; 3] {
    fn from(value: RgbValue) -> Self {
        [value.r, value.g, value.b]
    }
}
