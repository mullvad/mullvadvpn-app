plugins {
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.androidLibraryId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.map"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    composeOptions { kotlinCompilerExtensionVersion = Versions.kotlinCompilerExtensionVersion }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

dependencies {

    //Model
    implementation(project(Dependencies.Mullvad.modelLib))

    implementation(Dependencies.Compose.ui)
    implementation(Dependencies.Compose.foundation)

    implementation(Dependencies.AndroidX.lifecycleRuntimeKtx)
}
