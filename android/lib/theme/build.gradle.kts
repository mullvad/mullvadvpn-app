plugins {
    id("mullvad.android-library")
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.theme"

    buildFeatures { compose = true }
}

dependencies {
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.kotlin.stdlib)
}
