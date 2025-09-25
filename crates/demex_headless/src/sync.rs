pub trait DemexSync {
    type Sync;

    fn apply(&mut self, sync: Self::Sync);
    fn get_sync(&self) -> Self::Sync;
}
