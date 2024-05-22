object Dependencies {
    const val androidVolley = "com.android.volley:volley:${Versions.Android.volley}"
    const val commonsValidator = "commons-validator:commons-validator:${Versions.commonsValidator}"
    const val jodaTime = "joda-time:joda-time:${Versions.jodaTime}"
    const val junitApi = "org.junit.jupiter:junit-jupiter-api:${Versions.junit}"
    const val junitEngine = "org.junit.jupiter:junit-jupiter-engine:${Versions.junit}"
    const val junitParams = "org.junit.jupiter:junit-jupiter-params:${Versions.junit}"
    const val junitAndroidTestExtensions =
        "de.mannodermaus.junit5:android-test-core:${Versions.Android.junit}"
    const val junitAndroidTestCore =
        "de.mannodermaus.junit5:android-test-extensions:${Versions.Android.junit}"
    const val junitAndroidTestRunner =
        "de.mannodermaus.junit5:android-test-runner:${Versions.Android.junit}"
    const val konsist = "com.lemonappdev:konsist:${Versions.konsist}"
    const val leakCanary = "com.squareup.leakcanary:leakcanary-android:${Versions.leakCanary}"
    const val mockkWebserver = "com.squareup.okhttp3:mockwebserver:${Versions.mockWebserver}"
    const val turbine = "app.cash.turbine:turbine:${Versions.turbine}"
    const val billingClient = "com.android.billingclient:billing-ktx:${Versions.billingClient}"

    object AndroidX {
        const val appcompat = "androidx.appcompat:appcompat:${Versions.AndroidX.appcompat}"
        const val coreKtx = "androidx.core:core-ktx:${Versions.AndroidX.coreKtx}"
        const val fragmentTestning =
            "androidx.fragment:fragment-testing:${Versions.AndroidX.fragment}"
        const val lifecycleRuntimeKtx =
            "androidx.lifecycle:lifecycle-runtime-ktx:${Versions.AndroidX.lifecycle}"
        const val lifecycleViewmodelKtx =
            "androidx.lifecycle:lifecycle-viewmodel-ktx:${Versions.AndroidX.lifecycle}"
        const val lifecycleRuntimeCompose =
            "androidx.lifecycle:lifecycle-runtime-compose:${Versions.AndroidX.lifecycle}"
        const val lifecycleService =
            "androidx.lifecycle:lifecycle-service:${Versions.AndroidX.lifecycle}"
        const val espressoCore =
            "androidx.test.espresso:espresso-core:${Versions.AndroidX.espresso}"
        const val testCore = "androidx.test:core:${Versions.AndroidX.test}"
        const val testMonitor = "androidx.test:monitor:${Versions.AndroidX.testMonitor}"
        const val testRunner = "androidx.test:runner:${Versions.AndroidX.testRunner}"
        const val testRules = "androidx.test:rules:${Versions.AndroidX.test}"
        const val testUiAutomator =
            "androidx.test.uiautomator:uiautomator:${Versions.AndroidX.uiautomator}"
        const val testOrchestrator =
            "androidx.test:orchestrator:${Versions.AndroidX.testOrchestrator}"
    }

    object Arrow {
        const val core = "io.arrow-kt:arrow-core:${Versions.Arrow.base}"
        const val optics = "io.arrow-kt:arrow-optics:${Versions.Arrow.base}"
        const val opticsKsp = "io.arrow-kt:arrow-optics-ksp-plugin:${Versions.Arrow.base}"
    }

    object Compose {
        const val constrainLayout =
            "androidx.constraintlayout:constraintlayout-compose:${Versions.Compose.constrainLayout}"
        const val destinations =
            "io.github.raamcosta.compose-destinations:core:${Versions.Compose.destinations}"
        const val destinationsKsp =
            "io.github.raamcosta.compose-destinations:ksp:${Versions.Compose.destinations}"
        const val foundation =
            "androidx.compose.foundation:foundation:${Versions.Compose.foundation}"
        const val junit5 = "de.mannodermaus.junit5:android-test-compose:${Versions.Android.junit}"
        const val material3 = "androidx.compose.material3:material3:${Versions.Compose.material3}"
        const val testManifest = "androidx.compose.ui:ui-test-manifest:${Versions.Compose.base}"
        const val ui = "androidx.compose.ui:ui:${Versions.Compose.base}"
        const val uiTestManifest = "androidx.compose.ui:ui-test-manifest:${Versions.Compose.base}"
        const val uiTooling = "androidx.compose.ui:ui-tooling:${Versions.Compose.base}"
        const val uiToolingAndroidPreview =
            "androidx.compose.ui:ui-tooling-preview-android:${Versions.Compose.base}"
        const val uiToolingPreview =
            "androidx.compose.ui:ui-tooling-preview:${Versions.Compose.base}"
        const val uiUtil = "androidx.compose.ui:ui-util:${Versions.Compose.base}"
    }

    object Grpc {
        const val grpcOkHttp = "io.grpc:grpc-okhttp:${Versions.Grpc.grpcVersion}"
        const val grpcAndroid = "io.grpc:grpc-android:${Versions.Grpc.grpcVersion}"
        const val grpcKotlinStub = "io.grpc:grpc-kotlin-stub:${Versions.Grpc.grpcKotlinVersion}"
        const val protobufLite = "io.grpc:grpc-protobuf-lite:${Versions.Grpc.grpcVersion}"
        const val protobufKotlinLite =
            "com.google.protobuf:protobuf-kotlin-lite:${Versions.Grpc.protobufVersion}"
    }

    object Koin {
        const val core = "io.insert-koin:koin-core:${Versions.Koin.base}"
        const val android = "io.insert-koin:koin-android:${Versions.Koin.base}"
        const val compose = "io.insert-koin:koin-androidx-compose:${Versions.Koin.compose}"
        const val test = "io.insert-koin:koin-test:${Versions.Koin.base}"
    }

    object Kotlin {
        const val reflect = "org.jetbrains.kotlin:kotlin-reflect:${Versions.kotlin}"
        const val stdlib = "org.jetbrains.kotlin:kotlin-stdlib:${Versions.kotlin}"
        const val test = "org.jetbrains.kotlin:kotlin-test:${Versions.kotlin}"
    }

    object KotlinX {
        const val coroutinesCore =
            "org.jetbrains.kotlinx:kotlinx-coroutines-core:${Versions.kotlinx}"
        const val coroutinesAndroid =
            "org.jetbrains.kotlinx:kotlinx-coroutines-android:${Versions.kotlinx}"
        const val coroutinesTest =
            "org.jetbrains.kotlinx:kotlinx-coroutines-test:${Versions.kotlinx}"
    }

    object MockK {
        const val core = "io.mockk:mockk:${Versions.mockk}"
        const val android = "io.mockk:mockk-android:${Versions.mockk}"
    }

    object Mullvad {
        const val daemonGrpc = ":lib:daemon-grpc"
        const val vpnService = ":service"
        const val tileService = ":tile"

        const val commonLib = ":lib:common"
        const val endpointLib = ":lib:endpoint"
        const val modelLib = ":lib:model"
        const val resourceLib = ":lib:resource"
        const val talpidLib = ":lib:talpid"
        const val themeLib = ":lib:theme"
        const val commonTestLib = ":lib:common-test"
        const val billingLib = ":lib:billing"
        const val paymentLib = ":lib:payment"
        const val mapLib = ":lib:map"
        const val accountLib = ":lib:account"
        const val intentLib = ":lib:intent-provider"
        const val vpnPermissionLib = ":lib:vpn-permission"
    }

    object Plugin {
        // The aapt* plugin versions must be in sync with the android plugin version.
        // Required for Gradle metadata verification to work properly, see:
        // https://github.com/gradle/gradle/issues/19228
        const val aaptLinux = "com.android.tools.build:aapt2:${Versions.Plugin.androidAapt}:linux"
        const val aaptOsx = "com.android.tools.build:aapt2:${Versions.Plugin.androidAapt}:osx"
        const val aaptWindows =
            "com.android.tools.build:aapt2:${Versions.Plugin.androidAapt}:windows"
        const val android = "com.android.tools.build:gradle:${Versions.Plugin.android}"
        const val androidApplicationId = "com.android.application"
        const val androidLibraryId = "com.android.library"
        const val androidTestId = "com.android.test"
        const val dependencyCheck =
            "org.owasp:dependency-check-gradle:${Versions.Plugin.dependencyCheck}"
        const val dependencyCheckId = "org.owasp.dependencycheck"
        const val detektId = "io.gitlab.arturbosch.detekt"
        const val gradleVersionsId = "com.github.ben-manes.versions"
        const val junit5 = "de.mannodermaus.android-junit5"
        const val kotlin = "org.jetbrains.kotlin:kotlin-gradle-plugin:${Versions.kotlin}"
        const val kotlinAndroidId = "kotlin-android"
        const val kotlinParcelizeId = "kotlin-parcelize"
        const val ksp = "com.google.devtools.ksp"
        const val ktfmtId = "com.ncorti.ktfmt.gradle"
        const val playPublisher =
            "com.github.triplet.gradle:play-publisher:${Versions.Plugin.playPublisher}"
        const val playPublisherId = "com.github.triplet.play"
        const val protobufId = "com.google.protobuf"
    }
}
