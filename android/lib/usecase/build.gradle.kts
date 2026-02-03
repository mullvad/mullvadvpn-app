plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.junit5.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.usecase"

    buildFeatures { buildConfig = true }
}

dependencies {
    // implementation(projects.lib.ui.resource)
    implementation(projects.lib.common)
    implementation(projects.lib.grpc)
    implementation(projects.lib.model)
    implementation(projects.lib.repository)

    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.kermit)
    // implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)

    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.junit.jupiter.api)
    testImplementation(libs.junit.jupiter.params)
    testImplementation(libs.turbine)
    testImplementation(projects.lib.commonTest)
    testRuntimeOnly(libs.junit.jupiter.engine)
}
