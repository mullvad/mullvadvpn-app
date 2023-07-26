plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.common"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        targetSdk = Versions.Android.targetSdkVersion
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions { jvmTarget = Versions.jvmTarget }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        baseline = file("lint-baseline.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

dependencies {
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.talpidLib))

    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)
}
