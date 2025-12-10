use crate::pool::PoolType;

#[derive(Debug, Clone)]
pub enum PoolItemName {
    /// The pool item has it's own name
    String(String),

    /// The pool item doesn't have its own name but rather
    /// references another pool item
    Reference(PoolType, u32),
}

impl From<String> for PoolItemName {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl PoolItemName {
    pub fn name(name: impl Into<String>) -> Self {
        Self::String(name.into())
    }

    pub fn reference(pool_type: PoolType, id: u32) -> Self {
        Self::Reference(pool_type, id)
    }
}

impl std::fmt::Display for PoolItemName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(name) => write!(f, "{}", name),
            Self::Reference(pool_type, id) => write!(f, "{:?} {}", pool_type, id),
        }
    }
}

#[derive(Debug)]
pub struct PoolItem {
    pub id: u32,
    pub name: PoolItemName,
}

impl PartialOrd for PoolItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for PoolItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for PoolItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PoolItem {}
