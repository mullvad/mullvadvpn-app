plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.vpnsettings.impl" }

dependencies {
    implementation(projects.lib.feature.anticensorship.api)
    implementation(projects.lib.feature.autoconnect.api)
    implementation(projects.lib.feature.serveripoverride.api)
    implementation(projects.lib.feature.vpnsettings.api)
    implementation(projects.lib.feature.personalvpn.api)
    implementation(projects.lib.navigation)
    implementation(projects.lib.repository)
    implementation(projects.lib.usecase)

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
}
