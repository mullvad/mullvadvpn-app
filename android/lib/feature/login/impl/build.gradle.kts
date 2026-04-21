plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.login.impl" }

dependencies {
    implementation(projects.lib.feature.home.api)
    implementation(projects.lib.feature.login.api)
    implementation(projects.lib.feature.problemreport.impl)
    implementation(projects.lib.feature.settings.api)
    implementation(projects.lib.pushNotification)
    implementation(projects.lib.repository)
    implementation(projects.lib.usecase)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
