import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    id("de.mannodermaus.android-junit5")
}

android {
    namespace = "net.mullvad.mullvadvpn.test.arch"
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

androidComponents {
    beforeVariants { variantBuilder -> variantBuilder.apply { enable = name != "release" } }
}

dependencies {
    testRuntimeOnly(libs.junit.jupiter.engine)

    testImplementation(libs.compose.ui.tooling.android.preview)
    testImplementation(libs.compose.destinations)
    testImplementation(libs.androidx.appcompat)
    testImplementation(libs.junit.jupiter.api)
    testImplementation(libs.konsist)
}
