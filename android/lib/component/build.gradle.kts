plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.shared.compose"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig { minSdk = Versions.minSdkVersion }

    buildFeatures { compose = true }

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
}

dependencies {
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.constrainlayout)
    implementation(libs.kotlin.stdlib)
    implementation(libs.compose.icons.extended)
    implementation(libs.androidx.ktx)
    implementation(projects.lib.resource)
    implementation(projects.lib.shared)
    implementation(projects.lib.theme)
    implementation(projects.lib.model)
}
