import utilities.appVersionProvider
import utilities.configureMissingTranslationCheck

plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
}

val appVersion = appVersionProvider.get()

android {
    namespace = "net.mullvad.mullvadvpn.lib.ui.resource"

    sourceSets {
        getByName("main") {
            when {
                appVersion.isDev -> {
                    res.directories += "src/main/res-dev"
                }
                appVersion.isAlpha -> {
                    res.directories += "src/main/res-alpha"
                }
                else -> {
                    res.directories += "src/main/res-release"
                }
            }
        }
    }

    lint { configureMissingTranslationCheck(project, appVersion) }
}

dependencies {
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.coresplashscreen)
    implementation(libs.compose.ui)
}
