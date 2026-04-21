plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.appinfo.impl" }

dependencies {
    implementation(projects.lib.feature.appinfo.api)
    implementation(projects.lib.repository)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
