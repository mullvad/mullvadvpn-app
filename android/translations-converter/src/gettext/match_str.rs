/// A helper macro to match a string to various prefix and suffix combinations.
macro_rules! match_str {
    (
        ( $string:expr )
        $( [$start:expr, $middle:ident, $end:expr] => $body:tt )*
        _ => $else:expr $(,)*
    ) => {
        $(
            if let Some($middle) = parse_line($string, $start, $end) {
                $body
            } else
        )* {
            $else
        }
    };
}
