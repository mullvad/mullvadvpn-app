use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

error_chain!{
    errors {
        Null {
            description("Input is null pointer")
        }
        NoEqual(s: String) {
            description("No equal sign in string")
            display("No equal sign in \"{}\"", s)
        }
    }
    foreign_links {
        InvalidContent(::std::str::Utf8Error);
    }
}


/// Parses a null-terminated C string array into a Vec<String> for safe usage.
///
/// Returns an Err if given a null pointer or if a string is not valid utf-8.
///
/// # Segfaults
///
/// Can cause the program to crash if the pointer array starting at `ptr` is not correctly null
/// terminated. Likewise, if any string pointed to is not properly null-terminated it may crash.
pub unsafe fn string_array(mut ptr: *const *const c_char) -> Result<Vec<String>> {
    if ptr.is_null() {
        Err(Error::from(ErrorKind::Null))
    } else {
        let mut strings = Vec::new();
        while !(*ptr).is_null() {
            let cstr = CStr::from_ptr(*ptr);
            strings.push(cstr.to_str()?.to_owned());
            ptr = ptr.offset(1);
        }
        Ok(strings)
    }
}


/// Parses a null-terminated array of C strings with "=" delimiters into a key-value map.
///
/// The input environment has to contain null-terminated strings containing at least
/// one equal sign ("="). Every string is split at the first equal sign and added to the map with
/// the first part being the key and the second the value.
///
/// If multiple entries have the same key, the last one will be in the result map.
///
/// # Segfaults
///
/// Uses `string_array` internally and will segfault for the same reasons as that function.
pub unsafe fn env(envptr: *const *const c_char) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for string in string_array(envptr)? {
        let mut iter = string.splitn(2, "=");
        let key = iter.next().unwrap();
        let value = iter.next().ok_or(Error::from(ErrorKind::NoEqual(string.clone())))?;
        map.insert(key.to_owned(), value.to_owned());
    }
    Ok(map)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;
    use std::ptr;

    #[test]
    fn string_array_null() {
        let result = unsafe { string_array(ptr::null()) };
        assert_matches!(result, Err(Error(ErrorKind::Null, _)));
    }

    #[test]
    fn string_array_empty() {
        let ptr_arr = [ptr::null()];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert!(result.is_empty());
    }

    #[test]
    fn string_array_no_space_trim() {
        let test_str = " foobar \0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!([" foobar "], &result[..]);
    }

    #[test]
    fn string_array_two_strings() {
        let test_str1 = "foobar\0";
        let test_str2 = "barbaz\0";
        let ptr_arr = [test_str1 as *const _ as *const c_char,
                       test_str2 as *const _ as *const c_char,
                       ptr::null()];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(["foobar", "barbaz"], &result[..]);
    }

    #[test]
    fn env_one_value() {
        let test_str = "var_a=value_b\0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, result.len());
        assert_eq!(Some("value_b"), result.get("var_a").map(|s| &s[..]));
    }

    #[test]
    fn env_no_equal() {
        let test_str = "foobar\0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { env(&ptr_arr as *const *const c_char) };
        assert_matches!(result, Err(Error(ErrorKind::NoEqual(_), _)));
    }

    #[test]
    fn env_double_equal() {
        let test_str = "foo=bar=baz\0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let env = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, env.len());
        assert_eq!(Some("bar=baz"), env.get("foo").map(|s| &s[..]));
    }

    #[test]
    fn env_two_same_key() {
        let test_str1 = "foo=123\0";
        let test_str2 = "foo=abc\0";
        let ptr_arr = [test_str1 as *const _ as *const c_char,
                       test_str2 as *const _ as *const c_char,
                       ptr::null()];
        let env = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, env.len());
        assert_eq!(Some("abc"), env.get("foo").map(|s| &s[..]));
    }
}
