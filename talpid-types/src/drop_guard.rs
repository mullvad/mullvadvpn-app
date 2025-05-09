/// Create a value that executes function `F` when dropped.
#[must_use = "Should be assigned to a variable and implicitly or explicitly dropped"]
pub fn on_drop<F: FnOnce()>(f: F) -> OnDrop<F> {
    OnDrop(Some(f))
}

/// A type that executes a function when dropped.
///
/// The default type of `F` is a boxed function. It's also [Send] in order to be useful with async.
/// If you need a less restrictive type for `F` you can specify it explicitly.
pub struct OnDrop<F: FnOnce() = Box<dyn FnOnce() + Send>>(Option<F>);

impl<F: FnOnce()> OnDrop<F> {
    /// Map the wrapped function into some other function.
    pub fn map<F2: FnOnce()>(mut self, map: impl FnOnce(F) -> F2) -> OnDrop<F2> {
        let f = self.0.take();
        OnDrop(f.map(map))
    }

    /// A drop guard that does nothing
    pub fn noop() -> Self {
        OnDrop(None)
    }
}

impl<F: FnOnce() + Send + 'static> OnDrop<F> {
    /// Box the inner function to erase its type.
    pub fn boxed(self) -> OnDrop {
        self.map(|f| Box::new(f) as Box<_>)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f()
        }
    }
}
