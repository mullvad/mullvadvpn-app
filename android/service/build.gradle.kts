import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import utilities.FlavorDimensions
import utilities.Flavors
import utilities.appVersionProvider
import utilities.isReleaseBuild

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.junit5.android)
    alias(libs.plugins.mullvad.test)
}

val appVersion = appVersionProvider.get()

android {
    namespace = "net.mullvad.mullvadvpn.service"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig {
        minSdk = libs.versions.min.sdk.get().toInt()
        val shouldRequireBundleRelayFile = isReleaseBuild() && !appVersion.isDev
        buildConfigField(
            "Boolean",
            "REQUIRE_BUNDLED_RELAY_FILE",
            shouldRequireBundleRelayFile.toString(),
        )
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin {
        compilerOptions {
            jvmTarget = JvmTarget.fromTarget(libs.versions.jvm.target.get())
            allWarningsAsErrors = true
        }
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            isDefault = true
            // Not used for production builds.
            buildConfigField("String", "API_ENDPOINT", "\"\"")
            buildConfigField("String", "API_IP", "\"\"")
            buildConfigField("String", "SIGSUM_TRUSTED_PUBKEYS", "\"\"")
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api-app.devmole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.4\"")
            buildConfigField(
                "String",
                "SIGSUM_TRUSTED_PUBKEYS",
                "\"41ab8cc0fe1027757eda650545a09011ddfd1e4af278c520e19cc6513c850af8" +
                    ":f61f46a8a8fa4b1df038d36d78b97593f0f4f3b56857a6847399a028f765e0ce\"",
            )
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api-app.stagemole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.132\"")
            buildConfigField(
                "String",
                "SIGSUM_TRUSTED_PUBKEYS",
                "\"35809994d285fe3dd50d49c384db49519412008c545cb6588c138a86ae4c3284" +
                    ":9e05c843f17ed7225df58fdfd6ddcd65251aa6db4ad8ea63bd2bf0326e30577d\"",
            )
        }
    }

    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.grpc)
    implementation(projects.lib.endpoint)
    implementation(projects.lib.model)
    implementation(projects.lib.repository)
    implementation(projects.lib.talpid)

    implementation(libs.androidx.ktx)
    implementation(libs.androidx.lifecycle.service)
    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.koin)
    implementation(libs.koin.android)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.protobuf.kotlin.lite)

    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.junit.jupiter.api)
    testImplementation(libs.junit.jupiter.params)
    testImplementation(libs.turbine)
    testImplementation(projects.lib.commonTest)
    testRuntimeOnly(libs.junit.jupiter.engine)
}
