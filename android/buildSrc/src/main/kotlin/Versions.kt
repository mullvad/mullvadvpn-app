object Versions {
    const val commonsValidator = "1.7"
    const val jodaTime = "2.12.2"
    const val junit = "4.13.2"
    const val jvmTarget = "1.8"
    const val koin = "2.2.3"
    const val kotlin = "1.7.20"
    const val kotlinCompilerExtensionVersion = "1.3.2"
    const val kotlinx = "1.6.4"
    const val leakCanary = "2.10"
    const val mockk = "1.13.3"
    const val mockWebserver = "4.10.0"
    const val turbine = "0.12.1"

    object Android {
        const val compileSdkVersion = 33
        const val material = "1.7.0"
        const val minSdkVersion = 26
        const val targetSdkVersion = 33
        const val volley = "1.2.1"
    }

    object AndroidX {
        const val appcompat = "1.5.1"
        const val coreKtx = "1.9.0"
        const val constraintlayout = "2.1.4"
        const val coordinatorlayout = "1.2.0"
        const val espresso = "3.5.0"
        const val lifecycle = "2.5.1"
        const val fragment = "1.5.4"
        const val recyclerview = "1.2.1"
        const val junit = "1.1.4"
        const val test = "1.5.0"
        const val testMonitor = "1.6.0"
        const val testOrchestrator = "1.4.2"
        const val testRunner = "1.5.1"
        const val uiautomator = "2.2.0"
    }

    object Compose {
        const val base = "1.3.2"
        const val constrainLayout = "1.0.1"
        const val foundation = "1.3.1"
        const val material = "1.3.1"
        const val uiController = "0.28.0"
        const val viewModelLifecycle = "2.5.1"
    }

    object Plugin {
        // The androidAapt plugin version must be in sync with the android plugin version.
        const val android = "7.3.1"
        const val androidAapt = "$android-8691043"
        const val playPublisher = "3.7.0"
        const val dependencyCheck = "7.4.4"
        const val gradleVersions = "0.44.0"
    }
}
