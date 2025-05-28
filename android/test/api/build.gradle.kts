import utilities.BuildTypes
import utilities.FlavorDimensions
import utilities.Flavors

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlinx.serialization)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.api"
    compileSdk = libs.versions.compile.sdk.major.get().toInt()
    compileSdkMinor = libs.versions.compile.sdk.minor.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig { minSdk = libs.versions.min.sdk.get().toInt() }

    kotlin { compilerOptions { allWarningsAsErrors = true } }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

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

    // Required due to E2E tests having these flavors to avoid build errors.
    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) { dimension = FlavorDimensions.INFRASTRUCTURE }
        create(Flavors.STAGEMOLE) { dimension = FlavorDimensions.INFRASTRUCTURE }
    }
    buildFeatures { buildConfig = true }
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.apply { enable = name != BuildTypes.RELEASE }
    }
}

dependencies {
    implementation(projects.lib.endpoint)
    implementation(projects.lib.ui.tag)
    implementation(projects.lib.grpc)
    implementation(projects.lib.model)
    implementation(projects.test.common)

    implementation(libs.arrow)
    implementation(libs.androidx.test.core)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.junit.jupiter.engine)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.cio)
    implementation(libs.ktor.client.auth)
    implementation(libs.ktor.client.logging)
    implementation(libs.ktor.serialization.kotlinx.json)
    implementation(libs.ktor.client.content.negotiation)
    implementation(libs.ktor.client.resources)

    androidTestUtil(libs.androidx.test.orchestrator)
}
