Mullvad notes for creating an NSIS plug-in
==

Using `Visual Studio`, create a new DLL project.

Remove the x64 build configurations. Do this both for the solution and the project. 
  AMD64 support is spotty in NSIS and may not be supported at all when using NSIS via electron-builder.

Update `targetver.h` with an appropriate value for `_WIN32_WINNT`.

Disable /SafeSEH (it's under Linker -> Advanced).

Add the parent of the plugin SDK folder as an include directory for header files (it's under C/C++ -> General).

Add the plugin SDK folder as an include directory for linker libraries (it's under Linker -> General).

Add `pluginapi-x86-unicode.lib` to the set of linker libraries (it's under Linker -> Input).

Add `libc.lib` as an ignored linker library. `pluginapi-x86-unicode.lib` has a reference to `libc.lib`, which is
  apparently not needed. The setting is under Linker -> Input.

Change to static CRT linkage for the Release configuration (it's under C/C++ -> Code Generation).
