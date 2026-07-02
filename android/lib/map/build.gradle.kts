plugins {
    alias(libs.plugins.mullvad.android.library)
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
    implementation(projects.lib.ui.theme)

    implementation(libs.androidx.lifecycle.runtime)
    implementation(libs.compose.ui)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)

    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)
    implementation(libs.kermit)
}
