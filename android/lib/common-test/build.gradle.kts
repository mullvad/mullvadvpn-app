plugins { alias(libs.plugins.mullvad.android.library) }

android {
    namespace = "net.mullvad.mullvadvpn.lib.common.test"

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
}

dependencies {
    implementation(libs.kotlin.test)
    implementation(libs.kotlinx.coroutines.test)
    implementation(libs.junit.jupiter.api)
}
