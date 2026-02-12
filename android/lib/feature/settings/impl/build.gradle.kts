plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.settings.impl"
    ksp { arg("compose-destinations.moduleName", "settings") }
}

dependencies {
    implementation(projects.lib.repository)
    implementation(projects.lib.feature.apiaccess.impl)
    implementation(projects.lib.feature.appearance.impl)
    implementation(projects.lib.feature.appinfo.impl)
    implementation(projects.lib.feature.daita.impl)
    implementation(projects.lib.feature.multihop.impl)
    implementation(projects.lib.feature.notification.impl)
    implementation(projects.lib.feature.problemreport.impl)
    implementation(projects.lib.feature.splittunneling.impl)
    implementation(projects.lib.feature.vpnsettings.impl)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(libs.protobuf.kotlin.lite)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
