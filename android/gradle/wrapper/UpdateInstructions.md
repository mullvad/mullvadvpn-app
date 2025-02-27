# Update Gradle Wrapper

1. Go to the [releases page](https://gradle.org/releases/) and read up on the release notes that you want to update to.

2. Go to the [release checksums page](https://gradle.org/release-checksums/) and copy the `Binary-only (-bin) ZIP Checksum` for the version you want to update to.

3. Run the following command and replace the VERSION & SHA256 with the correct version and sha256 that you've retrieved from step 1.

```
./gradlew wrapper --gradle-version=VERSION --gradle-distribution-sha256-sum SHA256 && ./gradlew wrapper
```
