plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
}

android {
    namespace = "net.mullvad.mullvadvpn.test.arch"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig { minSdk = Versions.Android.minSdkVersion }

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
    testRuntimeOnly(Dependencies.junitEngine)

    testImplementation(Dependencies.Compose.uiToolingAndroidPreview)
    testImplementation(Dependencies.Compose.destinations)
    testImplementation(Dependencies.AndroidX.appcompat)
    testImplementation(Dependencies.junitApi)
    testImplementation(Dependencies.konsist)
}
