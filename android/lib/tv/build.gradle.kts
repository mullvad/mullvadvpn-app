plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.tv"

    buildFeatures { compose = true }
}

dependencies {
    implementation(libs.kotlin.stdlib)
    implementation(libs.androidx.tv)
    implementation(libs.androidx.activity.compose)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.repository)
    implementation(projects.lib.theme)
    implementation(projects.lib.ui.component)

    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)
}
