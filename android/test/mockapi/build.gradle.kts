plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.test.mockapi"
    compileSdk = Versions.compileSdkVersion

    defaultConfig {
        minSdk = Versions.minSdkVersion
        testApplicationId = "net.mullvad.mullvadvpn.test.mockapi"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        testInstrumentationRunnerArguments["runnerBuilder"] =
            "de.mannodermaus.junit5.AndroidJUnit5Builder"
        targetProjectPath = ":app"

        missingDimensionStrategy(FlavorDimensions.BILLING, Flavors.OSS)
        missingDimensionStrategy(FlavorDimensions.INFRASTRUCTURE, Flavors.PROD)

        testInstrumentationRunnerArguments.putAll(
            mapOf(
                "clearPackageData" to "true",
            )
        )
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

    kotlinOptions { jvmTarget = Versions.jvmTarget }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts += setOf(
                // Fixes packaging error caused by: jetified-junit-*
                "META-INF/LICENSE.md",
                "META-INF/LICENSE-notice.md"
            )
        }
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
    implementation(projects.lib.endpoint)
    implementation(projects.test.common)

    implementation(libs.androidx.test.core)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(libs.androidx.test.monitor)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.kermit)
    implementation(libs.jodatime)
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
