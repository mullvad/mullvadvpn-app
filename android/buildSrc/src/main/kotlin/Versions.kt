object Versions {
    const val commonsValidator = "1.7"
    const val jodaTime = "2.12.5"
    const val junit = "4.13.2"
    const val jvmTarget = "1.8"
    const val koin = "3.4.1"
    const val kotlin = "1.8.21"
    const val kotlinCompilerExtensionVersion = "1.4.7"
    const val kotlinx = "1.7.1"
    const val leakCanary = "2.11"
    // Make sure the following issue has been fixed before upgrading mockk:
    // https://github.com/mockk/mockk/issues/1035
    const val mockk = "1.13.3"
    const val mockWebserver = "4.11.0"
    const val turbine = "0.13.0"

    object Android {
        const val compileSdkVersion = 33
        const val material = "1.9.0"
        const val minSdkVersion = 26
        const val targetSdkVersion = 33
        const val volley = "1.2.1"
    }

    object AndroidX {
        const val appcompat = "1.6.1"
        const val coreKtx = "1.9.0"
        const val constraintlayout = "2.1.4"
        const val coordinatorlayout = "1.2.0"
        const val espresso = "3.5.1"
        const val lifecycle = "2.6.1"
        const val fragment = "1.5.7"
        const val recyclerview = "1.3.0"
        const val junit = "1.1.4"
        const val test = "1.5.0"
        const val testMonitor = "1.6.1"
        const val testOrchestrator = "1.4.2"
        const val testRunner = "1.5.2"
        const val uiautomator = "2.2.0"
    }

    object Compose {
        const val base = "1.4.3"
        const val composeCollapsingToolbar = "2.3.5"
        const val constrainLayout = "1.0.1"
        const val foundation = base
        const val material3 = "1.0.1"
        const val uiController = "0.30.1"
        const val viewModelLifecycle = "2.6.1"
    }

    object Plugin {
        // The androidAapt plugin version must be in sync with the android plugin version.
        const val android = "7.4.2"
        const val androidAapt = "$android-8841542"
        const val playPublisher = "3.8.3"
        const val dependencyCheck = "8.2.1"
        const val gradleVersions = "0.46.0"
        const val ktfmt = "0.12.0"
    }
}
