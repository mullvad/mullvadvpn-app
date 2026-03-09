plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.deleteaccount.impl" }

dependencies {
    implementation(projects.lib.repository)
    implementation(projects.lib.feature.login.impl)
    implementation(projects.lib.feature.login.api)
    implementation(projects.lib.feature.deleteaccount.api)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
