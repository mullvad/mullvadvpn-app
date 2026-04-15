## Debugging the native Rust libraries in Android Studio with LLDB

1. In `gradle.properties` or in your `$HOME/.gradle/gradle.properties` file set the following:

```
mullvad.app.build.keepDebugSymbols=true
mullvad.app.build.replaceRustPathPrefix=false
```

2. In Android Studio, go to `Run -> Edit configurations...`
3. Make sure the `app` configuration is selected.
4. In the `Debugger` tab, select `Dual (Java + Native)`
5. Start debugging the app as usual from Android Studio.
6. Click the `View Breakpoints...` icon in the debug view.
7. Click `+ -> Symbolic Breakpoints` and enter a function name in the `Symbol name` field.

Android Studio should now break on the function you selected.

NOTE: When running LLDB, Android Studio can sometimes get into a state where it will try to
connect to the debugger when running the app normally, which blocks the app from starting.
To fix this run `adb shell am clear-debug-app`.

### Setting line breakpoints in Rust code by clicking the gutter in Android Studio

By default, the normal debugging support in Android Studio such as clicking the gutter to set line breakpoints only
work when debugging C/C++ and not Rust code.

To fix this, install the following Android Studio plugin: https://github.com/kl/gutter-breakpoints-rust-plugin

### Improving LLDB interaction with common Rust types

Rust ships with the `rust-lldb` shell script (https://github.com/rust-lang/rust/blob/main/src/etc/rust-lldb) which
improves the LLDB experience when debugging Rust code, for example by being able to see the text value when
inspecting a `String` value.

To enable this when debugging in Android Studio, first find out your rustc sysroot by running
`rustc --print sysroot`. Then go to `Debugger -> LLDB Startup Commands` and enter the following
two commands:

```
command script import SYSROOT_PATH/lib/rustlib/etc/lldb_lookup.py
command source SYSROOT_PATH/lib/rustlib/etc/lldb_commands
````
