pub struct WithId<T, I: Copy + Clone> {
    inner: T,
    id: I,
}

impl<T, I: Copy + Clone> WithId<T, I> {
    pub fn map<F>(f: F) -> impl Fn(T) -> Self
    where
        F: Fn(&T) -> I,
    {
        move |inner| Self {
            id: f(&inner),
            inner,
        }
    }
}

impl<T, I: Copy + Clone> WithId<T, I> {
    pub fn new(inner: T, id: I) -> Self {
        Self { inner, id }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn id(&self) -> &I {
        &self.id
    }
}

impl<T: Clone, I: Copy + Clone> Clone for WithId<T, I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: self.id.clone(),
        }
    }
}

impl<T: Copy, I: Copy + Clone> Copy for WithId<T, I> {}
