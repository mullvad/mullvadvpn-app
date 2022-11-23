import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties
import java.util.Properties

plugins {
    id(Dependencies.Plugin.androidTestId)
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.e2e"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        targetSdk = Versions.Android.targetSdkVersion
        testApplicationId = "net.mullvad.mullvadvpn.test.e2e"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        targetProjectPath = ":app"

        fun Properties.addRequiredPropertyAsBuildConfigField(name: String) {
            val value = getProperty(name) ?: throw GradleException("Missing property: $name")
            buildConfigField(
                type = "String",
                name = name,
                value = "\"$value\""
            )
        }

        Properties().apply {
            load(project.file("e2e.properties").inputStream())
            addRequiredPropertyAsBuildConfigField("API_BASE_URL")
            addRequiredPropertyAsBuildConfigField("API_VERSION")
        }

        fun MutableMap<String, String>.addOptionalPropertyAsArgument(name: String) {
            val value = rootProject.properties.getOrDefault(name, null) as? String
                ?: gradleLocalProperties(rootProject.projectDir).getProperty(name)

            if (value != null) {
                put(name, value)
            }
        }

        testInstrumentationRunnerArguments += mutableMapOf<String, String>().apply {
            put("clearPackageData", "true")
            addOptionalPropertyAsArgument("valid_test_account_token")
            addOptionalPropertyAsArgument("invalid_test_account_token")
        }
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

configure<org.owasp.dependencycheck.gradle.extension.DependencyCheckExtension> {
    // Skip the lintClassPath configuration, which relies on many dependencies that has been flagged
    // to have CVEs, as it's related to the lint tooling rather than the project's compilation class
    // path. The alternative would be to suppress specific CVEs, however that could potentially
    // result in suppressed CVEs in project compilation class path.
    skipConfigurations = listOf("lintClassPath")
    suppressionFile = "$projectDir/e2e-suppression.xml"
}

dependencies {
    implementation(Dependencies.AndroidX.testCore)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(Dependencies.AndroidX.testMonitor)
    implementation(Dependencies.AndroidX.testOrchestrator)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testRules)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.androidVolley)
    implementation(Dependencies.junit)
    implementation(Dependencies.Kotlin.stdlib)
}
