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

val localScreenshotPath = "$buildDir/reports/androidTests/connected/screenshots"
val deviceScreenshotPath = "/sdcard/Pictures/Screenshots"

tasks.register("createDeviceScreenshotDir", Exec::class) {
    executable = android.adbExecutable.toString()
    args = listOf("shell", "mkdir", "-p", deviceScreenshotPath)
}

tasks.register("createLocalScreenshotDir", Exec::class) {
    executable = "mkdir"
    args = listOf("-p", localScreenshotPath)
}

tasks.register("clearDeviceScreenshots", Exec::class) {
    executable = android.adbExecutable.toString()
    args = listOf("shell", "rm", "-r", deviceScreenshotPath)
}

tasks.register("fetchScreenshots", Exec::class) {
    executable = android.adbExecutable.toString()
    args = listOf("pull", "$deviceScreenshotPath/.", localScreenshotPath)

    dependsOn(tasks.getByName("createLocalScreenshotDir"))
    finalizedBy(tasks.getByName("clearDeviceScreenshots"))
}

tasks.whenTaskAdded {
    if (name == "connectedDebugAndroidTest") {
        dependsOn(tasks.getByName("createDeviceScreenshotDir"))
        finalizedBy(tasks.getByName("fetchScreenshots"))
    }
}

dependencies {
    implementation(Dependencies.AndroidX.testCore)
    implementation(Dependencies.AndroidX.testOrchestrator)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testRules)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.Kotlin.stdlib)
}
