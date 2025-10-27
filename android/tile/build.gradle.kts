import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "net.mullvad.mullvadvpn.tile"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig { minSdk = libs.versions.min.sdk.get().toInt() }

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
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.repository)
    implementation(projects.lib.talpid)

    implementation(libs.koin)
    implementation(libs.koin.android)

    implementation(libs.androidx.appcompat)
    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
}
