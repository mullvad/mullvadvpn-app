plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android {
    namespace = "net.mullvad.mullvadvpn.feature.home.impl"
    ksp { arg("compose-destinations.moduleName", "home") }
}

dependencies {
    implementation(projects.lib.map)
    implementation(projects.lib.payment)
    implementation(projects.lib.pushNotification)
    implementation(projects.lib.repository)
    implementation(projects.lib.tv)
    implementation(projects.lib.usecase)
    implementation(projects.lib.feature.account.impl)
    implementation(projects.lib.feature.addtime.impl)
    implementation(projects.lib.feature.anticensorship.impl)
    implementation(projects.lib.feature.appinfo.impl)
    implementation(projects.lib.feature.daita.impl)
    implementation(projects.lib.feature.login.impl)
    implementation(projects.lib.feature.multihop.impl)
    implementation(projects.lib.feature.redeemvoucher.impl)
    implementation(projects.lib.feature.serveripoverride.impl)
    implementation(projects.lib.feature.settings.impl)
    implementation(projects.lib.feature.splittunneling.impl)
    implementation(projects.lib.feature.vpnsettings.impl)

    implementation(libs.androidx.animation)
    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(libs.compose.constrainlayout)
    implementation(libs.androidx.credentials) {
        // This dependency adds a lot of unused permissions to the app.
        // It is not used so let's exclude it.
        // Unfortunately, this is not possible to do using libs.version.toml
        // https://github.com/gradle/gradle/issues/26367#issuecomment-2120830998
        exclude("androidx.biometric", "biometric")
    }

    // Destinations
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)
}
