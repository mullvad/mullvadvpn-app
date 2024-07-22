object Versions {
    const val commonsValidator = "1.8.0"
    const val jodaTime = "2.12.7"
    const val junit = "5.10.2"
    const val jvmTarget = "17"
    const val kermit = "2.0.4"
    const val konsist = "0.15.1"
    const val kotlin = "2.0.0"
    const val kotlinx = "1.8.0"
    const val leakCanary = "2.13"
    const val mockk = "1.13.11"
    const val mockWebserver = "4.12.0"
    const val turbine = "1.0.0"
    const val billingClient = "6.2.0"

    object Android {
        const val compileSdkVersion = 34
        const val junit = "1.4.0"
        const val minSdkVersion = 26
        const val targetSdkVersion = 34
        const val volley = "1.2.1"
    }

    object AndroidX {
        const val activityCompose = "1.9.0"
        const val appcompat = "1.7.0"
        const val coreKtx = "1.13.1"
        const val coreSplashscreen = "1.1.0-rc01"
        const val espresso = "3.5.1"
        const val lifecycle = "2.8.3"
        const val fragmentTesting = "1.6.2"
        const val test = "1.5.0"
        const val testMonitor = "1.6.1"
        const val testOrchestrator = "1.4.2"
        const val testRunner = "1.5.2"
        const val uiautomator = "2.3.0"
    }

    object Arrow {
        const val base = "1.2.4"
    }

    object Compose {
        const val base = "1.7.0-beta05"
        const val destinations = "2.1.0-beta10"
        const val constrainLayout = "1.0.1"
        const val foundation = base
        const val material3 = "1.3.0-beta04"
    }

    object Grpc {
        const val grpcVersion = "1.65.1"
        const val grpcKotlinVersion = "1.4.1"
        const val protobufVersion = "4.27.2"
    }

    object Plugin {
        // The androidAapt plugin version must be in sync with the android plugin version.
        // Required for Gradle metadata verification to work properly, see:
        // https://github.com/gradle/gradle/issues/19228
        const val android = "8.3.0"
        const val androidAapt = "$android-10880808"
        const val playPublisher = "3.9.0"
        const val protobuf = "0.9.4"
        const val dependencyCheck = "10.0.1"
        const val detekt = "1.23.6"
        const val gradleVersions = "0.51.0"
        const val junit5 = "1.10.0.0"
        const val ktfmt = "0.17.0"
        // Ksp version is linked with kotlin version, find matching release here:
        // https://github.com/google/ksp/releases
        const val ksp = "${kotlin}-1.0.22"
    }

    object Koin {
        const val base = "3.5.6"
        const val compose = "3.5.6"
    }
}
