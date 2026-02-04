plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.compose)
}

android { namespace = "net.mullvad.mullvadvpn.core" }

dependencies {
    implementation(libs.androidx.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    testImplementation(libs.junit)
    implementation(libs.compose.ui)
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)

    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso)
}
