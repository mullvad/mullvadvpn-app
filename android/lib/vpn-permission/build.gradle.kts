plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.permission"
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
    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.commonLib))

    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)
    implementation(Dependencies.Arrow.core)
    implementation(Dependencies.jodaTime)
}
