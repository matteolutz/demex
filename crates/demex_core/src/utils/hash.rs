pub fn hash<T>(obj: T) -> u64
where
    T: std::hash::Hash,
{
    let mut hasher = std::hash::DefaultHasher::new();
    obj.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}
