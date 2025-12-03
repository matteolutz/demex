pub fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = 0;
    move |item| (f(i, item), i += 1).0
}

pub fn with_index_mut<T, F>(mut f: F) -> impl FnMut(&mut T) -> bool
where
    F: FnMut(usize, &mut T) -> bool,
{
    let mut i = 0;
    move |item| (f(i, item), i += 1).0
}
