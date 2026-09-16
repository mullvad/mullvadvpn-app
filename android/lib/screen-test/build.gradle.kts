plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
}

android { namespace = "net.mullvad.mullvadvpn.screen.test" }

dependencies {
    implementation(projects.lib.ui.theme)
    api(libs.androidx.ui.test)
    implementation(libs.androidx.ui.test.accessibility)
    // Accessibility Test Framework brings hamcrest at runtime scope only, but its
    // suppression matcher API needs the type at compile time.
    compileOnly(libs.hamcrest.core)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.test.core)
    implementation(libs.junit.jupiter.api)
    implementation(libs.androidx.core)
}
