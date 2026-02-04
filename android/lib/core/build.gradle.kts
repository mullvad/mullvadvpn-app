plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.core"
}

dependencies {
    implementation(libs.androidx.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    testImplementation(libs.junit)
    implementation(libs.compose.destinations)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso)
}
