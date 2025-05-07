/// Create a value that executes function `F` when dropped.
#[must_use = "Should be assigned to a variable and implicitly or explicitly dropped"]
pub fn on_drop<F: FnOnce()>(f: F) -> OnDrop<F> {
    OnDrop(Some(f))
}

/// A type that executes a function when dropped.
pub struct OnDrop<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f()
        }
    }
}
