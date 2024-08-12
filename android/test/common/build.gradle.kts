plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.common"
    compileSdk = Versions.compileSdkVersion

    defaultConfig { minSdk = Versions.minSdkVersion }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions { jvmTarget = Versions.jvmTarget }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts += setOf(
                // Fixes packaging error caused by: jetified-junit-*
                "META-INF/LICENSE.md",
                "META-INF/LICENSE-notice.md"
            )
        }
    }
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.apply { enable = name != BuildTypes.RELEASE }
    }
}

dependencies {
    implementation(projects.lib.endpoint)

    implementation(libs.androidx.testCore)
    implementation(libs.androidx.testRunner)
    implementation(libs.androidx.testRules)
    implementation(libs.androidx.testUiAutomator)
    implementation(Dependencies.junitJupiterEngine)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)

    androidTestUtil(libs.androidx.testOrchestrator)
}
