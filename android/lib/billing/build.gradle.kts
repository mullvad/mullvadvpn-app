plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.junit5.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.billing"

    defaultConfig { testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner" }

    packaging {
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md",
                )
        }
    }

    lint { baseline = file("${rootProject.projectDir.absolutePath}/config/lint-baseline.xml") }
}

dependencies {
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)

    // Billing library
    implementation(libs.android.billingclient)

    // Model
    implementation(projects.lib.model)

    // Payment library
    implementation(projects.lib.payment)

    // Either
    implementation(libs.arrow)

    // Management service
    implementation(projects.lib.daemonGrpc)

    // Logger
    implementation(libs.kermit)

    // Test dependencies
    testRuntimeOnly(libs.junit.jupiter.engine)

    testImplementation(projects.lib.commonTest)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.junit.jupiter.api)
    testImplementation(libs.turbine)

    androidTestImplementation(projects.lib.commonTest)
    androidTestImplementation(libs.mockk.android)
    androidTestImplementation(libs.kotlin.test)
    androidTestImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.turbine)
    androidTestImplementation(libs.junit.jupiter.api)
    androidTestImplementation(libs.junit.jupiter.engine)
    androidTestImplementation(libs.androidx.espresso)
}
