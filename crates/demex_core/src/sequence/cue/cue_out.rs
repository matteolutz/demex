use serde::{Deserialize, Serialize};

use crate::{implement_set_property, sequence::cue::CueFadingFunction};

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum CueOutProperty {
    InFade,
    FadingFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CueOut {
    pub fade: f32,
    pub fading_function: CueFadingFunction,
}

implement_set_property! {
    for CueOut with CueOutProperty,

    InFade => fade as f32,
    FadingFunction => fading_function as CueFadingFunction
}
