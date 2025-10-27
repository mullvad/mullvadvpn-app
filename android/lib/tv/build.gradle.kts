import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.tv"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig { minSdk = libs.versions.min.sdk.get().toInt() }

    buildFeatures { compose = true }

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
    implementation(libs.kotlin.stdlib)
    implementation(libs.androidx.tv)
    implementation(libs.androidx.activity.compose)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(projects.lib.model)
    implementation(projects.lib.resource)
    implementation(projects.lib.repository)
    implementation(projects.lib.theme)
    implementation(projects.lib.ui.component)

    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)
}
