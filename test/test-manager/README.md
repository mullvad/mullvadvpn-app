# Writing tests for [MullvadVPN App](https://github.com/mullvad/mullvadvpn-app/)

The `test-manager` crate is where end-to-end tests for the [MullvadVPN
App](https://github.com/mullvad/mullvadvpn-app/) resides. The tests are located
in different modules under `test-manager/src/tests/`.

## Getting started

Tests are regular Rust functions! Except that they are also `async` and marked
with the `#[test_function]` attribute

```rust
#[test_function]
pub async fn test(
    rpc: ServiceClient,
    mut mullvad_client: mullvad_management_interface::ManagementServiceClient,
) -> Result<(), Error> {
    Ok(())
}
```

The `test_function` macro allows you to write tests for the MullvadVPN App in a
format which is very similiar to [standard Rust unit
tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html). A more
detailed writeup on how the `#[test_function]` macro works is given as a
doc-comment in [test_macro::test_function](./test_macro/src/lib.rs).

If a new module is created, make sure to add it in
`test-manager/src/tests/mod.rs`.

### UI/Graphical tests

It is possible to write tests for asserting graphical properties in the app, but
this is a slightly more involved process. GUI tests are written in `Typescript`,
and reside in the `gui/test/e2e` folder in the app repository. Packaging of
these tests is also done from the `gui/` folder.

Assuming that a graphical test `gui-test.spec` has been bundled correctly, it
can be invoked from any Rust function by calling
`test_manager::tests::ui::run_test(rpc:
.., params: ..) -> Result<ExecResult,
Error>`

```rust
// Run a UI test. Panic if any assertion in it fails!
test_manager::tests::ui::run_test(&rpc, &["gui-test.spec"]).await.unwrap()
```

# Configuring `test-manager`

`test-manager` uses a configuration file to keep track of available virtual machines it can use for testing purposes.

More details can be found in [this configuration format document](./docs/config.md).
