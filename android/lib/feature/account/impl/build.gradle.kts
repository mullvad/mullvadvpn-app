plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.mullvad.android.library.compose)
    alias(libs.plugins.kotlin.android)
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

    // CameraX core library
    val camerax_version = "1.6.0"
    implementation("androidx.camera:camera-core:${camerax_version}")
    implementation("androidx.camera:camera-camera2:${camerax_version}")
    implementation("androidx.camera:camera-lifecycle:${camerax_version}")
    implementation("androidx.camera:camera-view:${camerax_version}")

// ML Kit Barcode Scanning (Bundled version - does NOT require Play Services)
    implementation("com.google.mlkit:barcode-scanning:17.3.0")

    implementation("androidx.concurrent:concurrent-futures-ktx:1.2.0")
    implementation("com.google.guava:guava:33.0.0-android")
}
