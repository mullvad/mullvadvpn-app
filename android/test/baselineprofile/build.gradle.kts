import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.baselineprofile)
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

        testInstrumentationRunnerArguments += buildMap {
            put("clearPackageData", "true")

            // Add all properties starting with "mullvad.test.e2e" to the
            // testInstrumentationRunnerArguments
            properties.forEach {
                if (it.key.startsWith("mullvad.test.e2e")) {
                    put(it.key, it.value.toString())
                }
            }
        }
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

dependencies {
    implementation(projects.lib.ui.tag)
    implementation(libs.androidx.junit)
    implementation(libs.androidx.espresso)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.androidx.benchmark.macro.junit4)
}

androidComponents {
    onVariants { v ->
        val artifactsLoader = v.artifacts.getBuiltArtifactsLoader()
        v.instrumentationRunnerArguments.put(
            "targetAppId",
            v.testedApks.map { artifactsLoader.load(it)?.applicationId },
        )
    }
}
