plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

android {
    namespace = "net.mullvad.mullvadvpn.service"
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

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            isDefault = true
            // Not used for production builds.
            buildConfigField("String", "API_ENDPOINT", "\"\"")
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.devmole.eu\"")
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.stagemole.eu\"")
        }
    }

    packagingOptions {
        this.excludes.add("META-INF/*")
        resources { excludes.add("META-INF/*") }
    }
    packaging { resources { excludes.add("META-INF/*") } }

    buildFeatures { buildConfig = true }
}

dependencies {
    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.endpointLib))
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.talpidLib))
    implementation(project(Dependencies.Mullvad.daemonGrpc))
    implementation(project(Dependencies.Mullvad.accountLib))
    implementation(project(Dependencies.Mullvad.intentLib))

    implementation(Dependencies.jodaTime)
    implementation(Dependencies.AndroidX.lifecycleService)
    implementation(Dependencies.Koin.core)
    implementation(Dependencies.Koin.android)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)
    implementation(Dependencies.Arrow.core)
    implementation(Dependencies.AndroidX.coreKtx)
}
