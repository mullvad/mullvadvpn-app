plugins {
    id(Dependencies.Plugin.androidTestId)
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    compileSdkVersion(Versions.Android.compileSdkVersion)

    defaultConfig {
        minSdkVersion(Versions.Android.minSdkVersion)
        targetSdkVersion(Versions.Android.targetSdkVersion)
        testApplicationId = "net.mullvad.mullvadvpn.e2e"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    testOptions {
        execution = "ANDROIDX_TEST_ORCHESTRATOR"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    targetProjectPath = ":app"
}

dependencies {
    implementation(Dependencies.AndroidX.testCore)
    implementation(Dependencies.AndroidX.testOrchestrator)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.Kotlin.stdlib)
}
