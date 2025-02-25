plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.test.mockapi"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig {
        minSdk = Versions.minSdkVersion
        testApplicationId = "net.mullvad.mullvadvpn.test.mockapi"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        testInstrumentationRunnerArguments["runnerBuilder"] =
            "de.mannodermaus.junit5.AndroidJUnit5Builder"
        targetProjectPath = ":app"

        missingDimensionStrategy(FlavorDimensions.BILLING, Flavors.OSS)
        missingDimensionStrategy(FlavorDimensions.INFRASTRUCTURE, Flavors.PROD)

        testInstrumentationRunnerArguments.putAll(mapOf("clearPackageData" to "true"))
    }

    flavorDimensions += FlavorDimensions.BILLING

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
    }

    testOptions { execution = "ANDROIDX_TEST_ORCHESTRATOR" }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        allWarningsAsErrors = true
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md",
                )
        }
    }
}

dependencies {
    implementation(projects.lib.endpoint)
    implementation(projects.test.common)

    implementation(libs.androidx.test.core)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(libs.androidx.test.monitor)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.kermit)
    implementation(Dependencies.junitJupiterApi)
    implementation(Dependencies.junit5AndroidTestExtensions)
    implementation(Dependencies.junit5AndroidTestRunner)
    implementation(libs.kotlin.stdlib)
    implementation(libs.mockkWebserver)

    androidTestUtil(libs.androidx.test.orchestrator)

    // Needed or else the app crashes when launched
    implementation(Dependencies.junit5AndroidTestCompose)
    implementation(libs.compose.material3)

    // Need these for forcing later versions of dependencies
    implementation(libs.compose.ui)
    implementation(libs.androidx.activity.compose)
}
