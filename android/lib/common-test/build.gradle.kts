plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.common.test"
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

dependencies {
    implementation(Dependencies.Kotlin.test)
    implementation(Dependencies.KotlinX.coroutinesTest)
    implementation(Dependencies.junit)
    implementation("org.junit.jupiter:junit-jupiter-api:5.10.0")
}
