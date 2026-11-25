use crate::pool::{PoolError, PoolItem, PoolType};

pub trait PoolHelper {
    fn ensure_pool_type(
        &self,
        expected_pool_type: PoolType,
        pool_type: PoolType,
    ) -> Result<(), PoolError>;
}

pub trait Pool {
    fn get(&self, pool_type: PoolType, id: u32) -> Result<PoolItem, PoolError>;
    fn get_all(&self, pool_type: PoolType) -> Result<Vec<PoolItem>, PoolError>;
    fn set_name(&mut self, pool_type: PoolType, id: u32, name: String) -> Result<(), PoolError>;
}

impl<P: Pool> PoolHelper for P {
    fn ensure_pool_type(
        &self,
        expected_pool_type: PoolType,
        pool_type: PoolType,
    ) -> Result<(), PoolError> {
        if expected_pool_type != pool_type {
            Err(PoolError::PoolTypeMismatch(expected_pool_type, pool_type))
        } else {
            Ok(())
        }
    }
}
