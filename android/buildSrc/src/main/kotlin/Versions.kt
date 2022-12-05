object Versions {
    const val commonsValidator = "1.7"
    const val jodaTime = "2.10.14"
    const val junit = "4.13.2"
    const val jvmTarget = "1.8"
    const val koin = "2.2.3"
    const val kotlin = "1.7.20"
    const val kotlinCompilerExtensionVersion = "1.3.2"
    const val kotlinx = "1.6.4"
    const val leakCanary = "2.8.1"
    const val mockk = "1.12.3"
    const val turbine = "0.7.0"

    object Android {
        const val compileSdkVersion = 33
        const val material = "1.4.0"
        const val minSdkVersion = 26
        const val targetSdkVersion = 33
        const val volley = "1.2.1"
    }

    object AndroidX {
        const val appcompat = "1.3.1"
        const val coreKtx = "1.6.0"
        const val constraintlayout = "2.1.3"
        const val coordinatorlayout = "1.1.0"
        const val espresso = "3.3.0"
        const val lifecycle = "2.4.1"
        const val fragment = "1.4.1"
        const val recyclerview = "1.2.1"
        const val junit = "1.1.4"
        const val test = "1.4.0"
        const val uiautomator = "2.2.0"
    }

    object Compose {
        const val base = "1.1.1"
        const val viewModelLifecycle = "2.4.1"
        const val uiController = "0.23.1"
        const val constrainLayout = "1.0.1"
    }

    object Plugin {
        // The androidAapt plugin version must be in sync with the android plugin version.
        const val android = "7.3.1"
        const val androidAapt = "$android-8691043"
        const val playPublisher = "3.7.0"
        const val dependencyCheck = "7.4.0"
        const val gradleVersions = "0.44.0"
    }
}
