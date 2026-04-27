use demex_core::channel3::attribute::FixtureChannel3Attribute;
use gpui::Rgba;

pub fn additive_attribute_to_rgba(attr: FixtureChannel3Attribute, value: f32) -> Option<Rgba> {
    match attr {
        // Base RGB
        FixtureChannel3Attribute::ColorAddR => Some(Rgba {
            r: value,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddG => Some(Rgba {
            r: 0.0,
            g: value,
            b: 0.0,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddB => Some(Rgba {
            r: 0.0,
            g: 0.0,
            b: value,
            a: 0.0,
        }),

        // Secondary colors
        FixtureChannel3Attribute::ColorAddC => Some(Rgba {
            r: 0.0,
            g: value,
            b: value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddM => Some(Rgba {
            r: value,
            g: 0.0,
            b: value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddY => Some(Rgba {
            r: value,
            g: value,
            b: 0.0,
            a: 0.0,
        }),

        // Extended emitters (approximate!)
        FixtureChannel3Attribute::ColorAddRY => Some(Rgba {
            r: value,
            g: 0.5 * value,
            b: 0.0,
            a: 0.0,
        }), // amber
        FixtureChannel3Attribute::ColorAddGY => Some(Rgba {
            r: 0.5 * value,
            g: value,
            b: 0.0,
            a: 0.0,
        }), // lime
        FixtureChannel3Attribute::ColorAddGC => Some(Rgba {
            r: 0.0,
            g: value,
            b: 0.5 * value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddBC => Some(Rgba {
            r: 0.0,
            g: 0.5 * value,
            b: value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddBM => Some(Rgba {
            r: 0.5 * value,
            g: 0.0,
            b: value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddRM => Some(Rgba {
            r: value,
            g: 0.0,
            b: 0.5 * value,
            a: 0.0,
        }),

        // Whites
        FixtureChannel3Attribute::ColorAddW => Some(Rgba {
            r: value,
            g: value,
            b: value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddWW => Some(Rgba {
            r: value,
            g: 0.85 * value,
            b: 0.7 * value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorAddCW => Some(Rgba {
            r: 0.7 * value,
            g: 0.85 * value,
            b: value,
            a: 0.0,
        }),

        // UV (invisible → slight blue hint)
        FixtureChannel3Attribute::ColorAddUV => Some(Rgba {
            r: 0.2 * value,
            g: 0.0,
            b: 0.8 * value,
            a: 0.0,
        }),

        _ => None,
    }
}

pub fn subtractive_attribute_to_rgba_factor(
    attr: FixtureChannel3Attribute,
    value: f32,
) -> Option<Rgba> {
    match attr {
        FixtureChannel3Attribute::ColorSubR => Some(Rgba {
            r: 1.0 - value,
            g: 1.0,
            b: 1.0,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorSubG => Some(Rgba {
            r: 1.0,
            g: 1.0 - value,
            b: 1.0,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorSubB => Some(Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0 - value,
            a: 0.0,
        }),

        FixtureChannel3Attribute::ColorSubC => Some(Rgba {
            r: 1.0,
            g: 1.0 - value,
            b: 1.0 - value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorSubM => Some(Rgba {
            r: 1.0 - value,
            g: 1.0,
            b: 1.0 - value,
            a: 0.0,
        }),
        FixtureChannel3Attribute::ColorSubY => Some(Rgba {
            r: 1.0 - value,
            g: 1.0 - value,
            b: 1.0,
            a: 0.0,
        }),

        _ => None,
    }
}
