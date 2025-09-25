use std::collections::HashMap;

pub mod channel3;
pub mod effect;
pub mod effect2;
pub mod error;
pub mod gdtf;
pub mod group_master;
pub mod handler;
pub mod keyframe_effect;
pub mod layout;
pub mod patch;
pub mod presets;
pub mod selection;
pub mod sequence;
pub mod timing;
pub mod updatables;
pub mod value_source;

pub type EncoderChannels = Vec<(&'static str, HashMap<u64, Vec<String>>)>;
