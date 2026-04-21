plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.account.impl" }

dependencies {
    implementation(projects.lib.feature.account.api)
    implementation(projects.lib.feature.addtime.api)
    implementation(projects.lib.feature.addtime.impl)
    implementation(projects.lib.feature.deleteaccount.api)
    implementation(projects.lib.feature.login.api)
    implementation(projects.lib.feature.managedevices.api)
    implementation(projects.lib.feature.redeemvoucher.api)
    implementation(projects.lib.payment)
    implementation(projects.lib.repository)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
}
