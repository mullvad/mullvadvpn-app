plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.billing"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig {
        minSdk = Versions.minSdkVersion
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        allWarningsAsErrors = true
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md",
                )
        }
    }
}

dependencies {
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)

    // Billing library
    implementation(libs.android.billingclient)

    // Model
    implementation(projects.lib.model)

    // Payment library
    implementation(projects.lib.payment)

    // Either
    implementation(libs.arrow)

    // Management service
    implementation(projects.lib.daemonGrpc)

    // Logger
    implementation(libs.kermit)

    // Test dependencies
    testRuntimeOnly(Dependencies.junitJupiterEngine)

    testImplementation(projects.lib.commonTest)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(Dependencies.junitJupiterApi)
    testImplementation(libs.turbine)

    androidTestImplementation(projects.lib.commonTest)
    androidTestImplementation(libs.mockk.android)
    androidTestImplementation(libs.kotlin.test)
    androidTestImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.turbine)
    androidTestImplementation(Dependencies.junitJupiterApi)
    androidTestImplementation(Dependencies.junitJupiterEngine)
    androidTestImplementation(libs.androidx.espresso)
}
