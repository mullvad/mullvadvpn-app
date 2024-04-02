object Versions {
    const val commonsValidator = "1.8.0"
    const val jodaTime = "2.12.7"
    const val junit = "5.10.2"
    const val jvmTarget = "17"
    const val konsist = "0.14.0"
    const val kotlin = "1.9.22"
    const val kotlinCompilerExtensionVersion = "1.5.10"
    const val kotlinx = "1.8.0"
    const val leakCanary = "2.13"
    const val mockk = "1.13.10"
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
        const val appcompat = "1.6.1"
        const val coreKtx = "1.12.0"
        const val espresso = "3.5.1"
        const val lifecycle = "2.7.0"
        const val fragment = "1.6.2"
        const val test = "1.5.0"
        const val testMonitor = "1.6.1"
        const val testOrchestrator = "1.4.2"
        const val testRunner = "1.5.2"
        const val uiautomator = "2.3.0"
    }

    object Arrow {
        const val base = "1.2.3"
    }

    object Compose {
        const val destinations = "1.10.2"
        const val base = "1.6.3"
        const val constrainLayout = "1.0.1"
        const val foundation = base
        const val material3 = "1.2.1"
    }

    object Plugin {
        // The androidAapt plugin version must be in sync with the android plugin version.
        // Required for Gradle metadata verification to work properly, see:
        // https://github.com/gradle/gradle/issues/19228
        const val android = "8.3.0"
        const val androidAapt = "$android-10880808"
        const val playPublisher = "3.9.0"
        const val dependencyCheck = "9.0.9"
        const val detekt = "1.23.5"
        const val gradleVersions = "0.51.0"
        const val junit5 = "1.10.0.0"
        const val ktfmt = "0.17.0"
        // Ksp version is linked with kotlin version, find matching release here:
        // https://github.com/google/ksp/releases
        const val ksp = "${kotlin}-1.0.17"
    }

    object Koin {
        const val base = "3.5.3"
        const val compose = "3.5.3"
    }
}
