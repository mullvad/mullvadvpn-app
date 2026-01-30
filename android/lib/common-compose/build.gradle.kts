plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.lib.common.compose" }

dependencies {
    implementation(projects.lib.ui.resource)
    implementation(projects.lib.model)
    implementation(projects.lib.common)
    implementation(libs.arrow)
    implementation(libs.kermit)
}
