plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.location.impl"
    ksp { arg("compose-destinations.moduleName", "location") }
}

dependencies {
    implementation(projects.lib.ui.icon)
    implementation(projects.lib.repository)
    implementation(projects.lib.usecase)
    implementation(projects.lib.feature.customlist.impl)
    implementation(projects.lib.feature.daita.impl)
    implementation(projects.lib.feature.filter.impl)

    implementation(libs.compose.constrainlayout)
    implementation(libs.koin.compose)
    implementation(libs.arrow)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
