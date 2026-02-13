plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.account.impl"
    ksp { arg("compose-destinations.moduleName", "account") }
}

dependencies {
    implementation(projects.lib.repository)
    implementation(projects.lib.payment)
    implementation(projects.lib.feature.addtime.impl)
    implementation(projects.lib.feature.login.impl)
    implementation(projects.lib.feature.managedevices.impl)
    implementation(projects.lib.feature.redeemvoucher.impl)

    implementation(libs.koin.compose)
    implementation(libs.arrow)

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
