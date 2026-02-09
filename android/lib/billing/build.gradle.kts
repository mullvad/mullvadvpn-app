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
