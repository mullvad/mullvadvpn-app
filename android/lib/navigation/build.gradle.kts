plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.mullvad.unit.test)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.kotlin.parcelize)
}

android { namespace = "net.mullvad.mullvadvpn.navigation" }

dependencies {
    api(libs.androidx.navigation3.runtime)
    api(libs.androidx.navigation3.ui)
    implementation(libs.androidx.lifecycle.viewmodel.navigation3)
    implementation(libs.compose.material3.windowsizeclass)
    implementation(libs.androidx.adaptive.layout)
}
