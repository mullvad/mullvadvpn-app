plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.problemreport.impl"
    ksp { arg("compose-destinations.moduleName", "problemreport") }
}

dependencies {
    implementation(projects.lib.repository)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(projects.lib.feature.redeemvoucher.api)
    implementation(projects.lib.feature.problemreport.api)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
