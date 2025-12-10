use demex_core::pool::PoolType;

pub trait PoolTypeExt {
    fn to_string(&self) -> String;
}

impl PoolTypeExt for PoolType {
    fn to_string(&self) -> String {
        match self {
            Self::Preset(feature_group) => format!("{} Preset", feature_group),
            Self::Sequence => format!("Sequence"),
            Self::Executor => format!("Executor"),
            Self::Group => format!("Group"),
            Self::Macro => format!("Macro"),
        }
    }
}
