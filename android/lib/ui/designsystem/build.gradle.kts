plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.ui.designsystem"

    buildFeatures { compose = true }
}

dependencies {
    implementation(projects.lib.theme)
    implementation(projects.lib.model)
    implementation(projects.lib.ui.tag)

    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.material3)
    implementation(libs.compose.icons.extended)
}
