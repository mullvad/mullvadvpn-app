plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.common"
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
    beforeVariants { variantBuilder ->
        variantBuilder.apply { enable = name != BuildTypes.RELEASE }
    }
}

dependencies {
    implementation(project(Dependencies.Mullvad.endpointLib))

    implementation(Dependencies.AndroidX.testCore)
    implementation(Dependencies.AndroidX.testRunner)
    implementation(Dependencies.AndroidX.testRules)
    implementation(Dependencies.AndroidX.testUiAutomator)
    implementation(Dependencies.junitEngine)
    implementation(Dependencies.Kotlin.stdlib)

    androidTestUtil(Dependencies.AndroidX.testOrchestrator)
}
