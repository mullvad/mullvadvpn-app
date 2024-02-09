plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.model"
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
    implementation(project(Dependencies.Mullvad.talpidLib))

    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    // Test dependencies
    testRuntimeOnly(Dependencies.junitEngine)

    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.junitApi)

    testImplementation(project(Dependencies.Mullvad.commonTestLib))
}
