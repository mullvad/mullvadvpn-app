plugins {
    id("mullvad.android-library")
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.map"

    buildFeatures {
        compose = true
        buildConfig = true
    }
}

dependencies {
    implementation(projects.lib.model)

    implementation(libs.androidx.lifecycle.runtime)
    implementation(libs.compose.ui)
    implementation(libs.compose.foundation)
    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)
    implementation(libs.kermit)
}
