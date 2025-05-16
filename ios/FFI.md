# Rust and FFI

This document is meant to provide best practices and conventions to follow
when writing FFI code between Swift and Rust.

## String and Data types

- Whenever possible, try to take advantage of Swift's automatic String conversion
- When dealing with buffers, a pointer to the start of the buffer, and its length should be passed
- Swift will most of the time do the right thing by passing a pointer to the start of the buffer
> [!IMPORTANT]
> Always remember that the pointers passed to the FFI functions are **only valid for the lifetime of the call**
> If you need to keep the contents of the pointer around for longer, make sure to copy it.

### Examples

Assuming the following FFI function

```C
void ffi_function(const char *str);
void other_ffi_function(const uint8_t *address, uintptr_t address_len);
```

How to call it from Swift

```swift
let someString = "hello"
ffi_function(hello)

...

let data = someString.data(using: .utf8)!
let dataAsArray = data.map { $0 }
other_ffi_function(dataAsArray, UInt(data.count))

```

## Opaque types

When dealing with opaque types, keep in mind that only types that are natively representable in C can be passed across the FFI boundary.
Fortunately, opaque types can be declared in such a way that they can be sent across the FFI to be reused later.

### Example

Here is a Rust type that accepts a pointer to an opaque Swift class sent as a `void *` type to Rust

```rust
#[repr(C)]
pub struct LoaderWrapperContext {
    // This pointer is a reference to a Swift object, and is only ever read by Rust.
    // It is used to call that Swift object across the FFI
    loader: *const c_void,
}

#[repr(C)]
pub struct SwiftLoaderWrapper(LoaderWrapperContext);
impl SwiftLoaderWrapper {
    pub fn new(context: LoaderWrapperContext) -> SwiftLoaderWrapper {
        SwiftLoaderWrapper(context)
    }
}
```

This allows to have a clean separation of the `unsafe` FFI boundary with a safe API in Rust

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_swift_loader_wrapper(
    loader: *const c_void,
) -> SwiftLoaderWrapper {
    let context = LoaderWrapperContext { loader };
    SwiftLoaderWrapper::new(context)
}

impl SwiftLoaderWrapper {
        pub fn safe_call(&self) -> Option<()> {
            let context = self.context_ref();
            Some(context.load_things())
        }

        fn context_ref(&self) -> &LoaderWrapperContext {
        &self.0
    }
}

impl LoaderWrapperContext {
    pub fn load_things() -> Option<()> {
        Some(unsafe { use_loader(self.loader) })
    }
}
```


Following the example above, here's how to provide an API from swift (That used from Rust in the previous example)

```swift
public func initLoaderWrapper(loader: Loader) -> SwiftLoaderWrapper {
    let rawLoader = Unmanaged.passUnretained(loader).toOpaque()
    return init_swift_loader_wrapper(rawLoader)
}

@_cdecl("use_loader")
func useLoader(rawLoader: UnsafeMutableRawPointer?) {
    guard let rawLoader else { return } }
    let loader = Unmanaged<Loader>.fromOpaque(rawLoader).takeUnretainedValue()
}
```

### More opaque types examples

### Digesting C style arrays
As there are no indications that some data type passed across the FFI boundary is a collection type,
the documentation has to be explicit about what's being sent.

As seen with the `other_ffi_function` example, a pointer to the start of the collection must be passed, alongside the number of elements in the collection.
If the collection is contiguous in memory, it can be read directly from the raw parts like so.
> [!IMPORTANT]
> Make sure to read the warning notice on `from_raw_parts`

```rust
unsafe fn generic_array_conversion_example<A>(raw_array: *const c_void, elems: usize) -> Vec<A>
where
    A: Sized,
{
    let raw_array: *mut *mut A = raw_array as _;
    // SAFETY: `raw_array` must be aligned, non-null and initialized for `count` reads
    let slice = unsafe { slice::from_raw_parts(raw_array, elems) };
    slice
        .iter()
        // SAFETY: Safety comment
        .map(|&ptr|  ... /* Turn the raw pointer into type A here */ )
        .collect()
}
```

### Boxing types

`Box`es are a very convenient way to transport opaque data around.
Chances are that you already have written some code that interacts with `Box` in one way or another.

Things to keep in mind when working with Box:
- Calling `Box::into_raw` *consumes* the boxed value.
    - In other words, the caller is responsible for making sure the memory managed by the `Box` instance that was turned to a raw pointer is taken care of.
- Calling `Box::from_raw` is the opposite operation and consumes the raw pointer, creating a `Box` in the process, and freeing the allocated memory for the raw pointer passed to it.
- Try not to move a pointer around after it's been boxed
- Always make sure to have matching calls to `into_raw` and `from_raw`

### Safety

Raw pointers are always *unsafe* to use in Rust.
This is why the separation of concerns is important, and you should always strive
to segregate the use of raw pointers and more idiomatic Rust code. Some guidelines to keep in mind

- Try to limit the scope of `unsafe` blocks to just where it's needed
- When working with `SAFETY: ` notices, describe the assumptions that are made at the call site, and the expectations placed upon the `unsafe` block
- Draw the line carefully between `safe` and `unsafe` Rust, try whenever possible to handle all the unsafe code in one single place (to make it easier to review) and build the safe layers upon that.
