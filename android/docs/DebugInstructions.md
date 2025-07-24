## Debugging the native libraries in Android Studio with LLDB

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
