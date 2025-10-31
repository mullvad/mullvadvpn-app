import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import utilities.BuildTypes

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.common"
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
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.apply { enable = name != BuildTypes.RELEASE }
    }
}

dependencies {
    implementation(projects.lib.endpoint)
    implementation(projects.lib.ui.tag)

    implementation(libs.androidx.test.core)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.junit.jupiter.engine)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)

    androidTestUtil(libs.androidx.test.orchestrator)
}
