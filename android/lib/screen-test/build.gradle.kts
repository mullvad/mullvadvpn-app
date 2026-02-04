plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
}

android { namespace = "net.mullvad.mullvadvpn.screen.test" }

dependencies {
    implementation(projects.lib.ui.theme)
    implementation(libs.junit5.android.test.compose)
}
