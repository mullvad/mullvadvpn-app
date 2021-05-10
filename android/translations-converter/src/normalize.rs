pub trait Normalize {
    /// Normalize the string value into a common format.
    ///
    /// Makes it possible to compare different representations of translation messages.
    fn normalize(&self) -> String;
}
