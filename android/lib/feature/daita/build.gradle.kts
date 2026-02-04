plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.compose)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.daita"

    ksp {
        arg("compose-destinations.moduleName", "daita")
    }
}

dependencies {
    implementation(projects.lib.core)
    implementation(projects.lib.ui.designsystem)
    implementation(projects.lib.ui.theme)
    implementation(projects.lib.ui.tag)
    implementation(projects.lib.ui.component)
    implementation(projects.lib.model)
    implementation(projects.lib.repository)

    implementation(libs.androidx.ktx)
    implementation(libs.koin.compose)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso)
}
