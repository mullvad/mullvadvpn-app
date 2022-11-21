import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties
import com.android.build.gradle.internal.tasks.factory.dependsOn
import java.io.FileInputStream
import java.util.Properties
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id(Dependencies.Plugin.androidApplicationId)
    id(Dependencies.Plugin.playPublisherId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
}

val repoRootPath = rootProject.projectDir.absoluteFile.parentFile.absolutePath
val extraAssetsDirectory = "${project.buildDir}/extraAssets"
val defaultChangeLogAssetsDirectory = "$repoRootPath/android/src/main/play/release-notes/"
val extraJniDirectory = "${project.buildDir}/extraJni"

val keystorePropertiesFile = file("${rootProject.projectDir}/keystore.properties")
val keystoreProperties = Properties()

if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    compileSdkVersion(Versions.Android.compileSdkVersion)

    defaultConfig {
        applicationId = "net.mullvad.mullvadvpn"
        minSdkVersion(Versions.Android.minSdkVersion)
        targetSdkVersion(Versions.Android.targetSdkVersion)
        versionCode = generateVersionCode()
        versionName = generateVersionName()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        val alwaysShowChangelog = gradleLocalProperties(rootProject.projectDir)
            .getProperty("ALWAYS_SHOW_CHANGELOG") ?: "false"
        buildConfigField(
            type = "boolean",
            name = "ALWAYS_SHOW_CHANGELOG",
            value = alwaysShowChangelog
        )
    }

    if (keystorePropertiesFile.exists()) {
        signingConfigs {
            create("release") {
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
                storeFile = file(keystoreProperties.getProperty("storeFile"))
                storePassword = keystoreProperties.getProperty("storePassword")
            }
        }

        buildTypes {
            getByName("release") {
                isMinifyEnabled = false
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }

    buildTypes {
        create("fdroid") {
            initWith(buildTypes.getByName("release"))
            isMinifyEnabled = false
            signingConfig = null
        }

        create("leakCanary") {
            initWith(buildTypes.getByName("debug"))
        }
    }

    sourceSets {
        getByName("main") {
            val changelogDir = gradleLocalProperties(rootProject.projectDir).getOrDefault(
                "OVERRIDE_CHANGELOG_DIR",
                defaultChangeLogAssetsDirectory
            )

            assets.srcDirs(extraAssetsDirectory, changelogDir)
            jniLibs.srcDirs(extraJniDirectory)
            java.srcDirs("src/main/kotlin/")
        }

        getByName("debug") {
            java.srcDirs("src/debug/kotlin/")
        }

        getByName("test") {
            java.srcDirs("src/test/kotlin/")
        }

        getByName("androidTest") {
            java.srcDirs("src/androidTest/kotlin/")
        }
    }

    buildFeatures {
        compose = true
    }

    composeOptions {
        kotlinCompilerVersion = Versions.kotlin
        kotlinCompilerExtensionVersion = Versions.kotlinCompilerExtensionVersion
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        freeCompilerArgs += "-Xopt-in=kotlin.RequiresOptIn"
        // Opt-in option for Koin annotation of KoinComponent.
    }

    tasks.withType<com.android.build.gradle.tasks.MergeSourceSetFolders> {
        dependsOn(getTasksByName("copyExtraAssets", true))
    }

    testOptions {
        unitTests.all { test ->
            test.testLogging {
                test.outputs.upToDateWhen { false }
                events("passed", "skipped", "failed", "standardOut", "standardError")
                showCauses = true
                showExceptions = true
            }
        }
    }

    packagingOptions {
        jniLibs.useLegacyPackaging = true

        // Fixes packaging error caused by: androidx.compose.ui:ui-test-junit4
        pickFirst("META-INF/AL2.0")
        pickFirst("META-INF/LGPL2.1")
    }

    project.tasks.preBuild.dependsOn("ensureJniDirectoryExist")
}

configure<org.owasp.dependencycheck.gradle.extension.DependencyCheckExtension> {
    // Skip the lintClassPath configuration, which relies on many dependencies that has been flagged
    // to have CVEs, as it's related to the lint tooling rather than the project's compilation class
    // path. The alternative would be to suppress specific CVEs, however that could potentially
    // result in suppressed CVEs in project compilation class path.
    skipConfigurations = listOf("lintClassPath")
}

tasks.withType<KotlinCompile>().all {
    kotlinOptions {
        allWarningsAsErrors = false

        kotlinOptions.freeCompilerArgs = listOf(
            "-Xuse-experimental=kotlinx.coroutines.ExperimentalCoroutinesApi",
            "-Xuse-experimental=kotlinx.coroutines.ObsoleteCoroutinesApi"
        )
    }
}

tasks.register("copyExtraAssets", Copy::class) {
    from("$repoRootPath/dist-assets")
    include("relays.json")
    into(extraAssetsDirectory)
}

tasks.register("ensureJniDirectoryExist") {
    doFirst {
        if (!file(extraJniDirectory).exists()) {
            throw GradleException("Missing JNI directory: $extraJniDirectory")
        }
    }
}

tasks.create("printVersion") {
    doLast {
        println("versionCode=${project.android.defaultConfig.versionCode}")
        println("versionName=${project.android.defaultConfig.versionName}")
    }
}

play {
    serviceAccountCredentials = file("play-api-key.json")
}

dependencies {
    implementation(Dependencies.androidMaterial)
    implementation(Dependencies.commonsValidator)
    implementation(Dependencies.AndroidX.appcompat)
    implementation(Dependencies.AndroidX.constraintlayout)
    implementation(Dependencies.AndroidX.coordinatorlayout)
    implementation(Dependencies.AndroidX.coreKtx)
    implementation(Dependencies.AndroidX.fragmentKtx)
    implementation(Dependencies.AndroidX.lifecycleRuntimeKtx)
    implementation(Dependencies.AndroidX.lifecycleViewmodelKtx)
    implementation(Dependencies.AndroidX.recyclerview)
    implementation(Dependencies.Compose.constrainLayout)
    implementation(Dependencies.Compose.foundation)
    implementation(Dependencies.Compose.viewModelLifecycle)
    implementation(Dependencies.Compose.material)
    implementation(Dependencies.Compose.uiController)
    implementation(Dependencies.Compose.ui)
    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Koin.core)
    implementation(Dependencies.Koin.coreExt)
    implementation(Dependencies.Koin.androidXFragment)
    implementation(Dependencies.Koin.androidXScope)
    implementation(Dependencies.Koin.androidXViewmodel)
    implementation(Dependencies.Kotlin.reflect)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    // Leak canary
    leakCanaryImplementation(Dependencies.leakCanary)

    // Test dependencies
    testImplementation(Dependencies.Koin.test)
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.junit)
    testImplementation(Dependencies.turbine)

    // UI test dependencies
    debugImplementation(Dependencies.AndroidX.fragmentTestning)
    debugImplementation(Dependencies.Compose.testManifest)
    androidTestImplementation(Dependencies.AndroidX.espressoContrib)
    androidTestImplementation(Dependencies.AndroidX.espressoCore)
    androidTestImplementation(Dependencies.Compose.junit)
    androidTestImplementation(Dependencies.Koin.test)
    androidTestImplementation(Dependencies.Kotlin.test)
    androidTestImplementation(Dependencies.MockK.android)
}
