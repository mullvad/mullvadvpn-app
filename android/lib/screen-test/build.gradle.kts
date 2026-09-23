plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
}

android { namespace = "net.mullvad.mullvadvpn.screen.test" }

dependencies {
    implementation(projects.lib.ui.theme)
    api(libs.androidx.ui.test.accessibility)
    implementation(libs.androidx.activity.compose)
    implementation(libs.junit.jupiter.api)
    implementation(libs.androidx.core)
}
