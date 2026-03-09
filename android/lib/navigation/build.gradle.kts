plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.navigation" }

dependencies {
    api(libs.androidx.navigation3.runtime)
    implementation(libs.androidx.lifecycle.viewmodel.navigation3)

    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
