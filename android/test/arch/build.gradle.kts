plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.test.arch"
    compileSdk = Versions.compileSdkVersion

    defaultConfig { minSdk = Versions.minSdkVersion }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions { jvmTarget = Versions.jvmTarget }

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
    testRuntimeOnly(Dependencies.junitJupiterEngine)

    testImplementation(libs.compose.ui.tooling.android.preview)
    testImplementation(libs.compose.destinations)
    testImplementation(libs.androidx.appcompat)
    testImplementation(Dependencies.junitJupiterApi)
    testImplementation(libs.konsist)
}
