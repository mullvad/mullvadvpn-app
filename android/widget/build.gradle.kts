plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.widget"
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
    buildFeatures { compose = true }

    // composeOptions { kotlinCompilerExtensionVersion = "1.5.15" }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.shared)
    implementation(projects.lib.talpid)

    implementation(libs.koin)
    implementation(libs.koin.android)

    implementation(libs.androidx.appcompat)
    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)

    // implementation(libs.glance.app.widget)
    // implementation(libs.glance.material)
    implementation(libs.glance.appwidget)
    implementation(libs.glance.material)
}
