plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.mullvad.unit.test)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.usecase"

    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.grpc)
    implementation(projects.lib.model)
    implementation(projects.lib.repository)

    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.kermit)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.androidx.annotation.jvm)
}
