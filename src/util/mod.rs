pub fn clone_option<T>(opt: &Option<T>) -> Option<T>
where
    T: Clone
{
    match opt {
        Some(t) => Some(t.clone()),
        None => None
    }
}