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
val extraJniDirectory = "${project.buildDir}/extraJni"

val keystorePropertiesFile = file("${rootProject.projectDir}/keystore.properties")
val keystoreProperties = Properties()

if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    compileSdkVersion(Versions.Android.compileSdkVersion)
    buildToolsVersion(Versions.Android.buildToolsVersion)

    defaultConfig {
        applicationId = "net.mullvad.mullvadvpn"
        minSdkVersion(Versions.Android.minSdkVersion)
        targetSdkVersion(Versions.Android.targetSdkVersion)
        versionCode = 21010099
        versionName = "2021.1"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
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
    }

    sourceSets {
        getByName("main") {
            assets.srcDirs(extraAssetsDirectory)
            jniLibs.srcDirs(extraJniDirectory)
            java.srcDirs("src/main/kotlin/")
        }

        getByName("test") {
            java.srcDirs("src/test/kotlin/")
        }

        getByName("androidTest") {
            java.srcDirs("src/androidTest/kotlin/")
        }
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
    include("api-ip-address.txt")
    into(extraAssetsDirectory)
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
    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Koin.core)
    implementation(Dependencies.Koin.coreExt)
    implementation(Dependencies.Koin.androidXFragment)
    implementation(Dependencies.Koin.androidXScope)
    implementation(Dependencies.Koin.androidXViewmodel)
    implementation(Dependencies.Kotlin.reflect)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    /* Test dependencies */
    testImplementation(Dependencies.Koin.test)
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.junit)

    /* UI test dependencies */
    debugImplementation(Dependencies.AndroidX.fragmentTestning)
    androidTestImplementation(Dependencies.AndroidX.espressoContrib)
    androidTestImplementation(Dependencies.AndroidX.espressoCore)
    androidTestImplementation(Dependencies.AndroidX.junit)
    androidTestImplementation(Dependencies.Koin.test)
    androidTestImplementation(Dependencies.Kotlin.test)
    androidTestImplementation(Dependencies.MockK.android)
}
