## Debugging the native libraries in Android Studio with LLDB

1. In Android Studio, go to `Run -> Edit configurations...`
2. Make sure the `app` configuration is selected.
3. In the `Debugger` tab, select `Dual (Java + Native)`
4. Start debugging the app as usual from Android Studio. The app should now stop on a SIGURG signal.
5. Select the `LLDB` tab in the debugger. Now you can set breakpoints etc, e.g.
   `breakpoint set -n open_tun`
6. Before continuing run `pro hand -p true -s false SIGURG`
7. Click `Resume Program` and the app will resume until the breakpoint is hit.

NOTE: When running LLDB, Android Studio can sometimes get into a state where it will try to
connect to the debugger when running the app normally, which blocks the app from starting.
To fix this run `adb shell am clear-debug-app`.
