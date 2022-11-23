# End-to-end (e2e) test module
## Overview
The tests in this module are end-to-end tests that rely on the publicly accessible Mullvad infrastucture and APIs. It's therefore required to provide a valid account token (not expired) that can be used to login, connect etc. It's also required to provide an invalid account token which for example is used for negative tests of the login flow. The invalid account token should not exist in the Mullvad infrastucture, however it must be at least 9 characters for some tests to properly run due to input validation.

## How to run the tests
### Locally
Set tokens in the below command and then execute the command in the `android` directory to run the tests on a local device:
```
./gradlew :test:e2e:connectedDebugAndroidTest \
    -Pvalid_test_account_token=XXXX \
    -Pinvalid_test_account_token=XXXX
```

For convenience, the tokens can also be set in `<REPO-ROOT>/android/local.properties` in the following way:
```
valid_test_account_token=XXXX
invalid_test_account_token=XXXX
```

It's also possible to provide the tokens to the test runner during test execution. However note that this requires [the APKs to be installed manually](https://developer.android.com/training/testing/instrumented-tests/androidx-test-libraries/runner#architecture).
```
adb shell 'CLASSPATH=$(pm path androidx.test.services) app_process / \
    androidx.test.services.shellexecutor.ShellMain am instrument -w \
    -e clearPackageData true \
    -e valid_test_account_token XXXX \
    -e invalid_test_account_token XXXX \
    -e targetInstrumentation net.mullvad.mullvadvpn.test.e2e/androidx.test.runner.AndroidJUnitRunner \
    androidx.test.orchestrator/.AndroidTestOrchestrator'
```

### Firebase Test Lab
Firebase Test Lab can be used to run the tests on vast collection of physical and virtual devices.

1. Setup the gcloud CLI by following the [official documentation](https://firebase.google.com/docs/test-lab/android/command-line).

2. Set tokens in the below command and then execute the command in the `android` directory to run the tests (on a Pixel 5e):
```
gcloud firebase test android run \
    --type instrumentation \
    --app ./android/app/build/outputs/apk/debug/app-debug.apk \
    --test ./android/test/e2e/build/outputs/apk/debug/e2e-debug.apk \
    --device model=redfin,version=30,locale=en,orientation=portrait \
    --use-orchestrator \
    --environment-variables clearPackageData=true,valid_test_account_token=XXXX,invalid_test_account_token=XXXX
```

If using gcloud via the docker image, the following can be executed in the `android` directory to run the tests (on a Pixel 5e):
```
docker run --rm --volumes-from gcloud-config -v ${PWD}:/android gcr.io/google.com/cloudsdktool/google-cloud-cli gcloud firebase test android run \
    --type instrumentation \
    --app ./android/app/build/outputs/apk/debug/app-debug.apk \
    --test ./android/test/e2e/build/outputs/apk/debug/e2e-debug.apk \
    --device model=redfin,version=30,locale=en,orientation=portrait \
    --use-orchestrator \
    --environment-variables clearPackageData=true,valid_test_account_token=XXXX,invalid_test_account_token=XXXX
```
