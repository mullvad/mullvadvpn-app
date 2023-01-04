# Mullvad VPN Electron app

This directory contains the files specific to the Electron app for the desktop platforms.

## Building and running the app

See the [build instructions](../BuildInstructions.md) for help building or running the app in
development mode.


## Automated tests

The app has unit tests and integration tests located in test/:
- **test/unit** (`npm run test`): Unit tests for specific parts of the app.
- **test/e2e**: End-to-end tests running against the UI.
  - **test/e2e/mocked** (`npm run e2e`): Tests running against the renderer process with a mocked
  main process (And therefore no daemon).
  - **test/e2e/installed** (`npm run e2e:installed <tests to run>`): Tests running against the app
  at its install path (See [Standalone test executable](#standalone-test-executable) for more info).
    - **test/e2e/installed/state-dependent** (`npm run e2e:installed state-dependent`): Tests
    requireing the daemon to be set into a specific state first.

### Standalone test executable

The tests in **test/e2e/installed** are run against the already installed app using the currently
running daemon. It's possible to run these tests on any machine with the app installed by running
```
npm run e2e:sequential installed/<test>
```
or without building by running
```
npm run e2e:sequential:no-build installed/<test>
```

It is also possible to build these tests along with all its dependencies into an executable that can
be run on any computer that has the app installed. The command for building the test executable is:
```
npm run build-test-executable
```
The executable for all platforms are outputted to `/dist/`.

The build is configured in `standalone-tests.ts`

The artifact can be run by either calling without arguments or with specific tests. A lot of the
tests are depending on the daemon already being in a specific state and will fail if it's not.
```
./mullvadvpn-app-e2e-tests-linux state-dependent/location.spec
```


## Other READMEs
- [scripts/README.md](scripts/README.md)
- [locales/README.md](locales/README.md)
- [assets/images/menubar-icons/README.md](assets/images/menubar-icons/README.md)
