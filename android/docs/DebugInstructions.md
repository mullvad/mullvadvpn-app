## Debugging the native libraries in Android Studio with LLDB

1. Make sure the native libraries have been built with debug symbols. If using the `build-apk.sh`
   script, run `SKIP_STRIPPING=yes ../build-apk.sh --dev-build`.
2. In Android Studio, go to `Run -> Edit configurations...`
3. Make sure the `app` configuration is selected.
4. In the `Debugger` tab, select `Dual (Java + Native)`
5. Start debugging the app as usual from Android Studio. The app should now stop on a SIGURG signal.
6. Select the `LLDB` tab in the debugger. Now you can set breakpoints etc, e.g.
   `breakpoint set -n open_tun`
7. Before continuing run `pro hand -p true -s false SIGURG`
8. Click `Resume Program` and the app will resume until the breakpoint is hit.
