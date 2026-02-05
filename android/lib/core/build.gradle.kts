plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.core" }

dependencies {
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
