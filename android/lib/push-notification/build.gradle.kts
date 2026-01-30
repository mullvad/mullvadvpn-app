plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
}

android { namespace = "net.mullvad.mullvadvpn.feature.pushnotifications" }

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.model)
    implementation(projects.lib.repository)
    implementation(projects.lib.ui.resource)

    implementation(libs.androidx.ktx)
    implementation(libs.androidx.lifecycle.service)
    implementation(libs.androidx.work.runtime.ktx)
    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.koin.android)
    implementation(libs.protobuf.kotlin.lite)
}
