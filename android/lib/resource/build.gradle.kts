plugins {
    id("mullvad.android-library")
    alias(libs.plugins.kotlin.parcelize)
}

android { namespace = "net.mullvad.mullvadvpn.lib.resource" }

dependencies {
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.coresplashscreen)
}
