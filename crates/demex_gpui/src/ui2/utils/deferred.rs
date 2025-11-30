///A deferred value that can be loaded asynchronously.
///
/// When [data](Deferred::data) is `None`, [loading](Deferred::loading) will always be set to `true`.
#[derive(Debug)]
pub struct Deferred<T> {
    data: Option<T>,
    loading: bool,
}

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self {
            data: None,
            loading: true,
        }
    }
}

impl<T> Deferred<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data(data: T) -> Self {
        Self {
            data: Some(data),
            loading: false,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn when_not_loading(&self) -> Option<&T> {
        if !self.loading {
            // when loading is set to false, data should be present
            Some(
                self.data
                    .as_ref()
                    .expect("Data should be present when loading is false"),
            )
        } else {
            None
        }
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
    }

    pub fn update(&mut self, data: T) {
        self.data = Some(data);
        self.loading = false;
    }
}
