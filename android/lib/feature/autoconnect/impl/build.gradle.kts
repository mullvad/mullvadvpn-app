plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.autoconnect.impl"
    ksp { arg("compose-destinations.moduleName", "autoconnect") }
}

dependencies {
    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(libs.compose.constrainlayout)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
