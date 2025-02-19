//add useful string trait to apply a function to a string
pub trait Apply {
    fn apply<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
        Self: Sized;
}

impl Apply for String {
    fn apply<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}