# Android Gradle Plugin 9.0.0 Upgrade Instructions

This document provides instructions for completing the Android Gradle Plugin upgrade to version 9.0.0.

## Changes Made

The following files have been updated:

1. **android/gradle/libs.versions.toml**
   - `android-gradle-plugin`: Updated from `8.13.2` to `9.0.0`
   - `android-gradle-aapt`: Updated from `14304508` to `14709011`

## Remaining Steps

### 1. Update Gradle Dependency Verification Metadata

The `android/gradle/verification-metadata.xml` file contains SHA256 checksums for all dependencies. It needs to be regenerated to include checksums for AGP 9.0.0 and its dependencies.

**To update the verification metadata:**

```bash
cd android/scripts
./lockfile --update
```

This script will:
- Remove old checksums from `verification-metadata.xml`
- Download all dependencies for AGP 9.0.0
- Generate new SHA256 checksums
- Update the verification metadata file

**Note**: This requires network access to Google's Maven repository and other dependency repositories.

### 2. Test the Build

After updating the verification metadata, test the build:

```bash
cd android
./gradlew clean build
```

### 3. Check for Deprecations and Breaking Changes

Review the [AGP 9.0.0 release notes](https://developer.android.com/build/releases/past-releases/agp-9-0-0-release-notes) for:
- Deprecated APIs that need to be updated
- Breaking changes that might affect the build
- New features that could be beneficial

### 4. Update CI/CD

Ensure that CI/CD pipelines have network access to download the new dependencies and can successfully build with AGP 9.0.0.

## Compatibility

- **Gradle Version**: The project is already using Gradle 9.3.0, which is compatible with AGP 9.0.0
- **JDK Version**: AGP 9.0.0 requires JDK 17, which is already configured in the project

## Troubleshooting

If you encounter issues:

1. **Dependency resolution errors**: Ensure all repositories (Google, Maven Central) are accessible
2. **Verification failures**: Run `./lockfile --update` to regenerate checksums
3. **Build failures**: Check the AGP release notes for breaking changes
4. **Cache issues**: Try running `./gradlew clean` and clearing Gradle caches with `rm -rf ~/.gradle/caches`

## References

- [Android Gradle Plugin 9.0.0 Release Notes](https://developer.android.com/build/releases/past-releases/agp-9-0-0-release-notes)
- [AGP 9.0.0 on Maven Repository](https://mvnrepository.com/artifact/com.android.tools.build/gradle/9.0.0)
- [AAPT2 9.0.0 on Maven Repository](https://mvnrepository.com/artifact/com.android.tools.build/aapt2)
