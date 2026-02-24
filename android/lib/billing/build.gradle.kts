plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.unit.test)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.billing"

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
    // play-services-location is a dependency of billing-ktx, but the billing-ktx specifies a
    // vulnerable version (19.0.0), so we need to explicitly use a later version here.
    implementation(libs.play.services.location)
    implementation(libs.android.billingclient)

    // Model
    implementation(projects.lib.model)

    // Payment library
    implementation(projects.lib.payment)

    // Either
    implementation(libs.arrow)

    // Management service
    implementation(projects.lib.grpc)

    // Logger
    implementation(libs.kermit)
}
