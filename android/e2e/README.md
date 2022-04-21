# End-to-end (e2e) test module
## Overview
The tests in this module are end-to-end tests that rely on the publicly accessible Mullvad infrastucture and APIs. It's therefore required to provide a valid token (not expired) that can be used to login, connect etc. It's also required to provide an invalid token which for example is used for negative tests of the login flow.

## How to run the tests
### Locally
Set proper tokens in the below command and then execute the command from the repository root to run the test sequence on a local device:
```
./android/gradlew -p ./android :e2e:connectedDebugAndroidTest -Pvalid_test_account_token=XXXX -Pinvalid_test_account_token=XXXX
```

For convenience, the tokens can also be set in `<REPO-ROOT>/android/local.properties` in the following way:
```
valid_test_account_token=XXXX
invalid_test_account_token=XXXX
```

It's also possible to provide the tokens to the test runner during test execution:
```
am instrument -e valid_test_account_token XXXX -e invalid_test_account_token XXXX # ...
```

### Firebase Test Lab
Firebase Test Lab can be used to execute tests sequence on vast collection of physical and virtual devices.

1. Setup the gcloud CLI by following the [official documentation](https://firebase.google.com/docs/test-lab/android/command-line).

2. Set proper tokens in the below command and then execute the command from the repository root to run the test sequence on a Pixel 5e.
```
gcloud firebase test android run \
--type instrumentation \
--app ./android/app/build/outputs/apk/debug/app-debug.apk \
--test ./android/e2e/build/outputs/apk/debug/e2e-debug.apk \
--device model=redfin,version=30,locale=en,orientation=portrait \
--use-orchestrator \
--environment-variables clearPackageData=true,valid_test_account_token=XXXX,invalid_test_account_token=XXXX
```

If using gcloud via the docker image, the following can be used to spawn the gcloud container with the current path mounted to `/app`:
```
docker run --rm --volumes-from gcloud-config -v ${PWD}:/app gcr.io/google.com/cloudsdktool/google-cloud-cli
```
