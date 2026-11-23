plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
}

android { namespace = "net.mullvad.mullvadvpn.feature.onboarding.impl" }

dependencies {
    implementation(projects.lib.feature.onboarding.api)
    implementation(projects.lib.feature.home.api)
    implementation(projects.lib.feature.addtime.api)
    implementation(projects.lib.repository)
    implementation(projects.lib.usecase)
    implementation(projects.lib.userPreferences)
    implementation(projects.lib.payment)

    implementation(libs.koin.compose)
    implementation(libs.arrow)
    implementation(libs.androidx.activity.compose)
    implementation(libs.accompanist.permissions)
}
