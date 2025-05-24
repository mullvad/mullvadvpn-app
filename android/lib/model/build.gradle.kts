@file:OptIn(ExperimentalKotlinGradlePluginApi::class)

import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)

    val powerAssert = libs.plugins.power.assert.get()
    kotlin(powerAssert.pluginId) version powerAssert.version.requiredVersion

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.model"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig {
        minSdk = Versions.minSdkVersion
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        allWarningsAsErrors = true
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

powerAssert {
    functions =
        listOf(
            "kotlin.assert",
            "kotlin.test.assertTrue",
            "kotlin.test.assertEquals",
            "kotlin.test.assertNull",
        )
    includedSourceSets = listOf("debugAndroidTest", "debugUnitTest", "release", "releaseUnitTest")
}

dependencies {
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    ksp(libs.arrow.optics.ksp)

    // Test dependencies
    testRuntimeOnly(Dependencies.junitJupiterEngine)

    testImplementation(libs.kotlin.test)
    testImplementation(Dependencies.junitJupiterApi)

    testImplementation(projects.lib.commonTest)
}
