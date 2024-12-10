plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "net.mullvad.mullvadvpn.service"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig { minSdk = Versions.minSdkVersion }

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

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            isDefault = true
            // Not used for production builds.
            buildConfigField("String", "API_ENDPOINT", "\"\"")
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.devmole.eu\"")
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.stagemole.eu\"")
        }
    }

    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.endpoint)
    implementation(projects.lib.model)
    implementation(projects.lib.shared)
    implementation(projects.lib.talpid)

    implementation(libs.androidx.ktx)
    implementation(libs.androidx.lifecycle.service)
    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.koin)
    implementation(libs.koin.android)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.jodatime)
}
