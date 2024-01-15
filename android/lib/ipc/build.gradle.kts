plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.ipc"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

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

dependencies {
    implementation(project(Dependencies.Mullvad.modelLib))

    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    androidTestImplementation(Dependencies.junitApi)
    androidTestImplementation(Dependencies.junitEngine)
    androidTestImplementation(Dependencies.AndroidX.testRunner)
    androidTestImplementation(Dependencies.Kotlin.test)
}
