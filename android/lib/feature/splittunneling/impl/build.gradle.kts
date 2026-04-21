plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.splittunneling.impl" }

dependencies {
    implementation(projects.lib.repository)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(projects.lib.feature.splittunneling.api)
}
