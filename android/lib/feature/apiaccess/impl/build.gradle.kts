plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.apiaccess.impl"
    ksp { arg("compose-destinations.moduleName", "apiaccess") }
}

dependencies {
    implementation(projects.lib.repository)
    implementation(projects.lib.navigation)

    implementation(libs.koin.compose)
    implementation(libs.arrow)

    // This dependency can be replaced when minimum SDK is 29 or higher.
    // It can then be replaced with InetAddress.isNumericAddress
    implementation(libs.commons.validator) {
        // This dependency has a known vulnerability
        // https://osv.dev/vulnerability/GHSA-wxr5-93ph-8wr9
        // It is not used so let's exclude it.
        // Unfortunately, this is not possible to do using libs.version.toml
        // https://github.com/gradle/gradle/issues/26367#issuecomment-2120830998
        exclude("commons-beanutils", "commons-beanutils")
    }

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
