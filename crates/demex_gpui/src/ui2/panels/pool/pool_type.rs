use demex_core::pool::PoolType;
use gpui::{App, Hsla};
use gpui_component::ActiveTheme;

pub trait PoolTypeExt {
    fn to_string(&self) -> String;
    fn to_short_string(&self) -> String;
    fn color(&self, cx: &App) -> Hsla;
}

impl PoolTypeExt for PoolType {
    fn to_string(&self) -> String {
        match self {
            Self::Preset(feature_group) => format!("{} Preset", feature_group),
            Self::Sequence => format!("Sequence"),
            Self::SequenceCue(seq_id) => format!("Sequence {} Cue", seq_id),
            Self::Executor => format!("Executor"),
            Self::Group => format!("Group"),
            Self::Macro => format!("Macro"),
        }
    }

    fn to_short_string(&self) -> String {
        match self {
            Self::Preset(feature_group) => feature_group.to_string(),
            Self::Sequence => format!("Seq"),
            Self::SequenceCue(seq_id) => format!("Seq {} Q", seq_id),
            Self::Executor => format!("Exec"),
            Self::Group => format!("Group"),
            Self::Macro => format!("Macro"),
        }
    }

    fn color(&self, cx: &App) -> Hsla {
        match self {
            Self::Group => cx.theme().red,
            Self::Preset(_) => cx.theme().blue,
            Self::Macro => cx.theme().yellow,
            Self::Sequence | Self::SequenceCue(_) => cx.theme().cyan,
            Self::Executor => cx.theme().green,
        }
    }
}
