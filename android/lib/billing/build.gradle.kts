plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
    id(Dependencies.Plugin.kotlinAndroidId)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.billing"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts += setOf(
                // Fixes packaging error caused by: jetified-junit-*
                "META-INF/LICENSE.md",
                "META-INF/LICENSE-notice.md"
            )
        }
    }
}

dependencies {
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    //Billing library
    implementation(Dependencies.billingClient)

    //Model
    implementation(project(Dependencies.Mullvad.modelLib))

    //Payment library
    implementation(project(Dependencies.Mullvad.paymentLib))

    // Test dependencies
    testRuntimeOnly(Dependencies.junitEngine)

    testImplementation(project(Dependencies.Mullvad.commonTestLib))
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.junitApi)
    testImplementation(Dependencies.turbine)

    androidTestImplementation(project(Dependencies.Mullvad.commonTestLib))
    androidTestImplementation(Dependencies.MockK.android)
    androidTestImplementation(Dependencies.Kotlin.test)
    androidTestImplementation(Dependencies.KotlinX.coroutinesTest)
    androidTestImplementation(Dependencies.turbine)
    androidTestImplementation(Dependencies.junitApi)
    androidTestImplementation(Dependencies.junitEngine)
    androidTestImplementation(Dependencies.AndroidX.espressoCore)
}
