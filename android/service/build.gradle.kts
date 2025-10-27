import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.junit5.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.service"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig {
        minSdk = libs.versions.min.sdk.get().toInt()
        val shouldRequireBundleRelayFile = isReleaseBuild() && !isDevBuild()
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
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api-app.devmole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.4\"")
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api-app.stagemole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.132\"")
        }
    }

    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
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
