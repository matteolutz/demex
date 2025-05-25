use strum::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EnumIter)]
pub enum RgbColorSpace {
    #[default]
    Srgb,

    AdobeRgb,
}

impl RgbColorSpace {
    pub fn rgb_to_xyz(self) -> nalgebra::SMatrix<f32, 3, 3> {
        match self {
            Self::Srgb => nalgebra::matrix![
                0.4124564,  0.3575761,  0.1804375;
                0.2126729,  0.7151522,  0.0721750;
                0.0193339,  0.1191920,  0.9503041
            ],
            Self::AdobeRgb => nalgebra::matrix![
                0.5767309,  0.1855540,  0.1881852;
                0.2973769,  0.6273491,  0.0752741;
                0.0270343,  0.0706872,  0.9911085
            ],
        }
    }

    pub fn xyz_to_rgb(self) -> nalgebra::SMatrix<f32, 3, 3> {
        match self {
            Self::Srgb => nalgebra::matrix![
                3.2404542, -1.5371385, -0.4985314;
                -0.9692660, 1.8760108,  0.0415560;
                0.0556434, -0.2040259,  1.0572252
            ],
            Self::AdobeRgb => nalgebra::matrix![
                2.0413690, -0.5649464, -0.3446944;
                -0.9692660,  1.8760108,  0.0415560;
                0.0134474, -0.1183897,  1.0154096;
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

    pub fn from_xy_bri(x: f32, y: f32, bri: f32, color_space: RgbColorSpace) -> Self {
        let big_y = bri;
        let big_x = (x / y) * big_y;
        let big_z = ((1.0 - x - y) / y) * big_y;

        let xyz_vector = nalgebra::vector![big_x, big_y, big_z];
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
}

impl From<RgbValue> for ecolor::Color32 {
    fn from(value: RgbValue) -> Self {
        let (r, g, b) = (value.r, value.g, value.b);
        ecolor::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}
