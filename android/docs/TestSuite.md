# Test Suite Overview

This document provides an overview of the Mullvad VPN Android application test suite, including architecture tests, end-to-end tests, and various testing utilities.

## Test Structure

The test suite is composed of the following test types:

### 1. Compose UI tests (`app/androidTest`)

Contains Compose UI tests that test specific UI screens, dialog and components.
These tests invoke composables directly with a given input, so they do not use repositories,
use-cases and viewmodels.

### 2. Unit tests (`app/test`, `service/test`, `lib/billing/test`, `lib/daemon-grpc/test`, `lib/repository/test`)

Contains unit tests that test specific use-cases, viewmodels and repositories, etc.

### 3. End-to-End Tests (`test/e2e/`)

Various tests that simulate a user interacting with the app using the Android UI automation framework.
The e2e tests require a real API backend and real relays that the tests can connect to. The tests can be set
to target different environments such as `prod` or `stagemole`.

__Note:__ some of the e2e tests are written with the assumption that `stagemole` is used,
and will fail if run on `prod`.

#### Example of E2E tests include:
- Testing that the user can successfully login.
- Testing that the user can connect to a relay.
- Testing that the user can still connect via an obfuscation method (e.g. Shadowsocks) if Wireguard is blocked at the network level.

#### E2E tests setup:
In order to run the e2e tests a few properties must be set in your `~/.gradle/properties.gradle` file:

```bash
# For running the e2e tests on the production backend
mullvad.test.e2e.prod.accountNumber.valid=INSERT_VALID_ACCOUNT_NUMBER_HERE
mullvad.test.e2e.prod.accountNumber.invalid=1111222233334444

# For running the e2e tests on the stagemole backend
mullvad.test.e2e.stagemole.partnerAuth=INSER_PARTNER_AUTH_TOKEN_HERE
mullvad.test.e2e.stagemole.accountNumber.invalid=1111222233334444

# For running e2e tests that require the RAAS router
# (see: mullvadvpn-app/ci/ios/test-router)
mullvad.test.e2e.config.raas.host=INSERT_RAAS_HOST_HERE
mullvad.test.e2e.config.raas.enable=true
```

### 4. Mock API Tests (`test/mockapi/`)

The mock-api tests are also E2E tests, with the difference being that the API responses are mocked to return pre-defined responses.
The main benefit of this is to speed up the tests as much as a possible as the normal E2E tests can be slow, and to test things
that are difficult to test with the real API, such as account expiry notifications being shown.

#### Example of mock-api tests include:
- Testing that the out-of-time notification is shown.
- Testing that the too many devices screen/flow works as expected.

### 5. Architecture Tests (`test/arch/`)

These tests ensure that the codebase follows architectural rules and conventions using the `Konsist` library.

#### Example of things being tested
- ViewModels must have `ViewModel` suffix
- All Compose previews are private functions and must start with `Preview`
- Ensures data classes only use immutable properties (`val` not `var`)

### 6. Detekt static analysis (`test/detekt/`)

Detekt is used for static analysis using the configuration in `config/detekt.yml` and `config/detekt-baseline.xml`.
The `test/detekt` directory contains custom detekt rules. Currently the only custom rule
checks that `Screen` and `Dialog` composable functions only use named arguments.

### 7. Android Lint static analysis

Android Lint is enabled for static analysis using the configuration in `config/lint.yml` and
`config/lint-baseline.xml`.`

### Other modules in the `test` directory

#### Baseline Profile Generation (`test/baselineprofile/`)

The baseline profile generation code is only used to generate baseline profiles and not to test the app, but the code lives
in the `test` directory because it needs to use the Android UI automation framework to interact with the app when generating
the baseline profile.

#### Common Test Utilities (`test/common/`)

This module contains various helpers and abstractions that make navigating and testing the app using
the Android UI automation framework easier.

It is used by the `e2e`, `mockapi` and `baselineprofile` modules.

It contains utilities such as:

- `findObjectByCaseInsensitiveText()`: Case-insensitive text search
- `findObjectWithTimeout()`: Robust element location with timeout
- `clickObjectAwaitCondition()`: Click and wait for condition
- `acceptVpnPermissionDialog()`: Handles the system VPN permission dialog

This module also has the `Page` class which is used as an abstraction to make it easier to create UI automator tests.
The following is an example of using `Page` in test:
```kotlin
@Test
fun testConnect() {
    // Given
    app.launchAndLogIn(accountTestRule.validAccountNumber)

    on<ConnectPage> { clickConnect() }

    device.acceptVpnPermissionDialog()

    on<ConnectPage> { waitForConnectedLabel() }
}
```

The `on` function asserts that page given in the generic argument is displayed (each page class implements an abstract method that checks if the page is displayed)
and then invokes the lambda that can call methods on the page.

## Running tests

### Unit tests
```bash
./gradlew testOssProdDebugUnitTest
```

### Compose UI tests
```bash
./gradlew :app:connectedOssDebugAndroidTest
```

### E2E tests
```bash
./gradlew :test:e2e:connectedPlayStagemoleDebugAndroidTest
```
The e2e tests can also be run using the `prod` backend (but some tests may fail because
they only work when running on the `stagemole` backend):
```bash
./gradlew :test:e2e:connectedPlayProdDebugAndroidTest
```

### Mock API tests
```bash
./gradlew :test:mockapi:connectedOssDebugAndroidTest
```

### Detekt
```bash
./gradlew detekt
```

### Android lint
```bash
./gradlew lint
```

### Architecture tests (Konsist)
```bash
./gradlew :test:arch:test --rerun-tasks
```

## Continuous Integration

Our CI runs via Github actions. The unit, arch, detekt, lint and Compose UI tests must all pass before
a pull request can be merged to `main`.

The end-to-end tests are not run for each pull request because they take too long to complete, but
are run multiple times every night in a separate Github action.

Some E2E tests are sometimes flaky, but we aim to always have a fully working test suite, so a flaky E2E test
is considered a bug that should be fixed.
