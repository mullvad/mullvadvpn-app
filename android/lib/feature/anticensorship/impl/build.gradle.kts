plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.anticensorship.impl" }

dependencies {
    implementation(projects.lib.feature.anticensorship.api)
    implementation(projects.lib.navigation)
    implementation(projects.lib.repository)
    implementation(projects.lib.ui.util)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
