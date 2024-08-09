plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.shared"
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
    implementation(project(Dependencies.Mullvad.resourceLib))
    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.daemonGrpc))
    implementation(project(Dependencies.Mullvad.modelLib))

    implementation(Dependencies.Arrow.core)
    implementation(Dependencies.kermit)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)
    implementation(Dependencies.jodaTime)

    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.junitApi)
    testImplementation(Dependencies.junitParams)
    testImplementation(Dependencies.turbine)
    testImplementation(project(Dependencies.Mullvad.commonTestLib))
    testRuntimeOnly(Dependencies.junitEngine)
}
