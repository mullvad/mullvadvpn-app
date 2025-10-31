plugins {
    id("mullvad.android-library")
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.ui.component"

    buildFeatures { compose = true }

    kotlin { compilerOptions { freeCompilerArgs.add("-XXLanguage:+WhenGuards") } }
}

dependencies {
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.theme)
    implementation(projects.lib.ui.tag)
    implementation(projects.lib.ui.designsystem)

    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.constrainlayout)
    implementation(libs.kotlin.stdlib)
    implementation(libs.compose.icons.extended)
    implementation(libs.androidx.ktx)
}
