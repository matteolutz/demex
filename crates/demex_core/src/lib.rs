use std::collections::HashMap;

use uuid::Uuid;

pub mod channel3;
pub mod color;
pub mod command;
pub mod dmx;
pub mod effect;
pub mod effect2;
pub mod engine;
pub mod event;
pub mod fixture;
pub mod group_master;
// pub mod headless;
pub mod input;
pub mod keyframe_effect;
pub mod layout;
pub mod patch;
pub mod pool;
pub mod presets;
pub mod selection;
pub mod sequence;
pub mod show;
pub mod state;
mod thread;
pub mod timing;
pub mod updatables;
pub mod utils;
pub mod value_source;

pub type FixtureTypeAndMode = (Uuid, String);
pub type EncoderChannels = Vec<(&'static str, HashMap<FixtureTypeAndMode, Vec<String>>)>;
