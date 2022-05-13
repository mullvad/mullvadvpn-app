/// A helper macro to match a string to various prefix and suffix combinations.
///
/// This macro can be used in a way that's similar to matching slices. It's possible to match an
/// input string to:
///
/// - a specified string;
/// - a string with a specified prefix;
/// - a string with a specified suffix;
/// - a string with a specified prefix and a specified suffix.
///
/// Multiple match patterns can be specified for the same match arm, as long as there are no
/// bindings for that match arm. When matching with prefixes and/or suffixes, the known parts of the
/// string can be removed, and the rest of the string is bound to a binding with a specified name.
///
/// The macro has a limitation where all match arm bodies must be separated by commas, even if the
/// body is inside braces (`{}`).
///
/// # Examples
///
/// When not using any bindings, multiple match patterns can be on the same match arm.
///
/// ```
/// # let input_string = "";
///
/// match_str! { (input_string.trim())
///     ["exact_string"] => {
///         println!("Exact match")
///     }, // Note: even though the body is enclosed by braces, a comma is still necessary
///     ["prefix", ..] | [.., "suffix"] => println!("Partial match"),
///     no_match => println!("Input {:?} did not match", no_match),
/// }
/// ```
///
/// If a match arm uses a binding, it must only have one match pattern.
///
/// ```
/// # let input_string = "";
///
/// match_str! { (input_string.trim())
///     ["prefix", string] => println!("Prefixed: {:?}", string),
///     [string, "suffix"] => println!("Suffixed: {:?}", string),
///     ["prefix", string, "suffix"] => println!("Prefixed and Suffixed: {:?}", string),
///     // The following does not work because the match arm has a binding and therefore can't have
///     // more than one pattern:
///     // ["prefix", string] | [string, "suffix"] => {
///     //     println!("Prefixed or Suffixed: {:?}", string)
///     // }
/// }
/// ```
///
/// # Implementation details
///
/// The macro starts by extracting the matched expression and binding it to a local variable. It
/// will then call itself recursively to build an `if`-`else` chain to match that variable
/// according to the desired prefix/suffix patterns.
///
/// When calling itself recursively, a `@match_str` marker is used to mark that the macro is
/// inside an inner call. The marker is follows by an initial state, which consists of three parts.
/// The first part is the condition expression, which is built by all the match patterns of that
/// arm. The second part is the binding for the input string. The third part is the binding used
/// for that match arm.
///
/// The third part of the state initially starts out empty, but is later replaced by either a
/// binding expression or a `@no_bindings` marker. The marker allows the condition to grow with
/// other patterns in the same match arm.
macro_rules! match_str {
    // Start of matching
    ( ($string:expr) $(|)* $( $match_body:tt )* ) => {
        {
            let string_to_match = $string;

            match_str!(@match_str((false), string_to_match) | $( $match_body )*)
        }
    };

    // Match a whole string
    (
        @match_str($conditions:tt, $input:ident $(, @no_bindings)*)
        | [$string:literal] $( $rest:tt )*
    ) => {
        match_str!(@match_str(($conditions || $input == $string), $input, @no_bindings) $( $rest )*)
    };

    // Match a string with a given prefix
    (
        @match_str($conditions:tt, $input:ident $(, @no_bindings)*)
        | [$prefix:literal, ..] $( $rest:tt )*
    ) => {
        match_str!(
            @match_str(($conditions || $input.starts_with($prefix)), $input, @no_bindings)
            $( $rest )*
        )
    };

    // Match a string with a given prefix and suffix
    (
        @match_str($conditions:tt, $input:ident $(, @no_bindings)*)
        | [$prefix:literal, .., $suffix:literal]
        $( $rest:tt )*
    ) => {
        match_str!(
            @match_str(
                ($conditions || ($input.starts_with($prefix) && $input.ends_with($suffix))),
                $input,
                @no_bindings
            )
            $( $rest )*
        )
    };

    // Match a string with a given prefix and suffix, binding the middle of the string, starting
    // after the prefix and ending before the suffix
    (
        @match_str($conditions:tt, $input:ident)
        | [$prefix:literal, $binding:ident, $suffix:literal] $( $rest:tt )*
    ) => {
        match_str!(
            @match_str(
                ($conditions || ($input.starts_with($prefix) && $input.ends_with($suffix))),
                $input,
                @binding $binding = &$input[$prefix.len()..($input.len()-$suffix.len())]
            )
            $( $rest )*
        )
    };

    // Final `else` body with a catch-all binding
    ( @match_str((false), $input:ident) | $binding:ident => $body:expr $(,)* ) => {
        {
            let $binding = $input;

            $body
        }
    };

    // Build `if` body
    (
        @match_str($conditions:tt, $input:ident, @no_bindings)
        => $body:expr , $(,)* $(|)* $( $rest:tt )*
    ) => {
        if $conditions {
            $body
        } else {
            match_str!(@match_str((false), $input) | $( $rest )*)
        }
    };

    // Build `if` body with a specified binding
    (
        @match_str($conditions:tt, $input:ident, @binding $binding:ident = $binding_expr:expr)
        => $body:expr , $(,)* $(|)* $( $rest:tt )*
    ) => {
        if $conditions {
            let $binding = $binding_expr;

            $body
        } else {
            match_str!(@match_str((false), $input) | $( $rest )*)
        }
    };
}
