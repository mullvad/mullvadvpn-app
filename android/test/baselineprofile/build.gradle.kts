import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import utilities.FlavorDimensions
import utilities.Flavors
import utilities.Variant
import utilities.baselineFilter
import utilities.matches

plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.baselineprofile)
    id("mullvad.utilities")
}

android {
    namespace = "net.mullvad.mullvadvpn.test.baselineprofile"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin {
        compilerOptions {
            jvmTarget = JvmTarget.fromTarget(libs.versions.jvm.target.get())
            allWarningsAsErrors = true
        }
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    defaultConfig {
        minSdk = 28
        targetSdk = libs.versions.target.sdk.get().toInt()
        targetProjectPath = ":app"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        testInstrumentationRunnerArguments += mapOf("clearPackageData" to "true")
    }

    targetProjectPath = ":app"

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField(
                type = "String",
                name = "INFRASTRUCTURE_BASE_DOMAIN",
                value = "\"mullvad.net\"",
            )
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField(
                type = "String",
                name = "INFRASTRUCTURE_BASE_DOMAIN",
                value = "\"stagemole.eu\"",
            )
        }
    }
    buildFeatures { buildConfig = true }
}

// This is the configuration block for the Baseline Profile plugin.
// You can specify to run the generators on a managed devices or connected devices.
baselineProfile { useConnectedDevices = true }

// Force okio version to 3.9.1 to fix 2.10.0 appearing in the verification metadata file.
// This is to avoid a osv-scanner complaining a about a vulnerability in okio 2.10.0.
// Gradle already upgrades okio 2.10.0 to 3.9.1, but it still ends up in the metadata file.
// If we update androidx.benchmark:benchmark-macro-junit4 we might be able to remove this.
configurations.all { resolutionStrategy { force("com.squareup.okio:okio:3.9.1") } }

dependencies {
    implementation(projects.lib.ui.tag)
    implementation(libs.androidx.junit)
    implementation(libs.androidx.espresso)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.androidx.benchmark.macro.junit4)
}

androidComponents {
    beforeVariants { it.enable = Variant(it.buildType, it.productFlavors).matches(baselineFilter) }
    onVariants { v ->
        val artifactsLoader = v.artifacts.getBuiltArtifactsLoader()
        v.instrumentationRunnerArguments.put(
            "targetAppId",
            v.testedApks.map { artifactsLoader.load(it)?.applicationId },
        )
    }
}
