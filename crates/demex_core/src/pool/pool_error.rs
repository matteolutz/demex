use crate::pool::PoolType;

#[derive(Debug)]
pub enum PoolError {
    PoolTypeMismatch(PoolType, PoolType),
    InvalidPoolType(PoolType),
    PoolItemNotFound(PoolType, u32),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::PoolTypeMismatch(expected, actual) => {
                write!(
                    f,
                    "Pool type mismatch: expected {:?}, got {:?}",
                    expected, actual
                )
            }
            PoolError::InvalidPoolType(pool_type) => {
                write!(f, "Invalid pool type: {:?}", pool_type)
            }
            PoolError::PoolItemNotFound(pool_type, id) => {
                write!(f, "Pool item not found: {:?} with id {}", pool_type, id)
            }
        }
    }
}

impl std::error::Error for PoolError {}
