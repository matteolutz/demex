use std::collections::HashMap;

pub mod channel3;
pub mod color;
pub mod command;
pub mod effect;
pub mod effect2;
pub mod engine;
pub mod fixture;
pub mod group_master;
pub mod headless;
pub mod input;
pub mod keyframe_effect;
pub mod layout;
pub mod patch;
pub mod presets;
pub mod selection;
pub mod sequence;
pub mod show;
pub mod timing;
pub mod updatables;
pub mod utils;
pub mod value_source;

pub type EncoderChannels = Vec<(&'static str, HashMap<u64, Vec<String>>)>;
