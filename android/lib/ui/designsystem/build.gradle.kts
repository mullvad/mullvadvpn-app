plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.ui.designsystem"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig { minSdk = libs.versions.min.sdk.get().toInt() }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = libs.versions.jvm.target.get()
        allWarningsAsErrors = true
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

dependencies {
    implementation(projects.lib.theme)
    implementation(projects.lib.model)
    implementation(projects.lib.ui.tag)

    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.icons.extended)
}
