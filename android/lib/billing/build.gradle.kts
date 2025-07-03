import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.junit5.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.billing"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig {
        minSdk = libs.versions.min.sdk.get().toInt()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
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
