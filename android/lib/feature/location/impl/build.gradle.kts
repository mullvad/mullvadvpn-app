plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.location.impl" }

dependencies {
    implementation(projects.lib.ui.icon)
    implementation(projects.lib.repository)
    implementation(projects.lib.usecase)
    implementation(projects.lib.feature.customlist.api)
    implementation(projects.lib.feature.daita.api)
    implementation(projects.lib.feature.filter.api)
    implementation(projects.lib.feature.location.api)

    implementation(libs.compose.constrainlayout)
    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
