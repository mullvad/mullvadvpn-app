plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.junit5.android)
}

android { namespace = "net.mullvad.mullvadvpn.lib.model" }

dependencies {
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    ksp(libs.arrow.optics.ksp)

    // Test dependencies
    testRuntimeOnly(libs.junit.jupiter.engine)

    testImplementation(libs.kotlin.test)
    testImplementation(libs.junit.jupiter.api)

    testImplementation(projects.lib.commonTest)
}
