plugins {
    id(Dependencies.Plugin.androidTestId)
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.mockapi"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        targetSdk = Versions.Android.targetSdkVersion
        testApplicationId = "net.mullvad.mullvadvpn.test.mockapi"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        targetProjectPath = ":app"

        testInstrumentationRunnerArguments.putAll(
            mapOf(
                "clearPackageData" to "true",
            )
        )
    }

    testOptions {
        execution = "ANDROIDX_TEST_ORCHESTRATOR"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
    }

    lint {
        abortOnError = true
        warningsAsErrors = true
    }
}

configure<org.owasp.dependencycheck.gradle.extension.DependencyCheckExtension> {
    // Skip the lintClassPath configuration, which relies on many dependencies that has been flagged
    // to have CVEs, as it's related to the lint tooling rather than the project's compilation class
    // path. The alternative would be to suppress specific CVEs, however that could potentially
    // result in suppressed CVEs in project compilation class path.
    skipConfigurations = listOf("lintClassPath")
    suppressionFile = "$projectDir/../test-suppression.xml"
}

dependencies {
    implementation(project(Projects.testCommon))
    implementation(project(Dependencies.Mullvad.endpointLib))

    implementation(Dependencies.AndroidX.testCore)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(Dependencies.AndroidX.testMonitor)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testRules)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.mockkWebserver)

    androidTestUtil(Dependencies.AndroidX.testOrchestrator)
}
