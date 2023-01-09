plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.common"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        targetSdk = Versions.Android.targetSdkVersion
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
    }
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.apply {
            enable = name != "release"
        }
    }
}

dependencies {
    implementation(Dependencies.AndroidX.testCore)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testRules)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.junit)
    implementation(Dependencies.Kotlin.stdlib)

    androidTestUtil(Dependencies.AndroidX.testOrchestrator)
}
