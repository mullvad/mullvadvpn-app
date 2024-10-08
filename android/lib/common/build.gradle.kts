plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.common"
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
        abortOnError = true
        warningsAsErrors = true
    }
}

dependencies {
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.talpid)

    implementation(libs.androidx.appcompat)
    implementation(libs.jodatime)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
}
