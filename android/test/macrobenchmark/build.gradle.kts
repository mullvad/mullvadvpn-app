plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.androidx.baselineprofile)
}

android {
    namespace = "net.mullvad.mullvadvpn.macrobenchmark"
    compileSdk = 35

    defaultConfig {
        minSdk = 26
        targetSdk = 35

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        // Disable background work during benchmarking
        testInstrumentationRunnerArguments["androidx.benchmark.junit4.SideEffectRunListener"] =
            "androidx.benchmark.junit4.SideEffectRunListener"
    }

    targetProjectPath = ":app"

    buildTypes {
        create(BuildTypes.BENCHMARK) {
            isDebuggable = true
            signingConfig = getByName(BuildTypes.DEBUG).signingConfig
            matchingFallbacks += listOf(BuildTypes.RELEASE)
        }
    }

    //    flavorDimensions += listOf(FlavorDimensions.BILLING, FlavorDimensions.INFRASTRUCTURE)
    //    productFlavors {
    //        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
    //        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
    //        create(Flavors.PROD) { dimension = FlavorDimensions.INFRASTRUCTURE }
    //    }

    // Enable the benchmark to run separately from the app process
    experimentalProperties["android.experimental.self-instrumenting"] = true

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        allWarningsAsErrors = true
    }
}

baselineProfile { useConnectedDevices = false }

dependencies {
    implementation(libs.androidx.junit)
    implementation(libs.androidx.espresso)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.androidx.benchmark.macro.junit4)
}

androidComponents {
    beforeVariants(selector().all()) {
        it.enable = it.name == "ossProdRelease" || it.name == "playProdRelease"
    }
}
