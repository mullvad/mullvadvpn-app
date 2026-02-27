plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.anticensorship.impl"
    ksp { arg("compose-destinations.moduleName", "anticensorship") }
}

dependencies {
    implementation(projects.lib.repository)
    implementation(projects.lib.navigation)
    implementation(projects.lib.ui.util)

    implementation(libs.koin.compose)
    implementation(libs.arrow)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
