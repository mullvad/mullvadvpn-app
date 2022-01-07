
import java.io.FileInputStream
import java.util.Properties
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("com.android.application")
    id("com.github.triplet.play")
    id("kotlin-android")
    id("kotlin-parcelize")
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
    compileSdkVersion(30)
    buildToolsVersion("30.0.3")

    defaultConfig {
        applicationId = "net.mullvad.mullvadvpn"
        minSdkVersion(26)
        targetSdkVersion(30)
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
        jvmTarget = "1.8"
        freeCompilerArgs += "-Xopt-in=kotlin.RequiresOptIn"
        // Opt-in option for Koin annotation of KoinComponent.
    }

    applicationVariants.forEach { variant ->
        variant.mergeAssetsProvider.configure{
            dependsOn(task("copyExtraAssets"))
        }
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
    val espressoVersion: String by rootProject.extra
    val fragmentVersion: String by rootProject.extra
    val koinVersion: String by rootProject.extra
    val kotlinVersion: String by rootProject.extra
    val mockkVersion: String by rootProject.extra

    implementation("androidx.appcompat:appcompat:1.3.1")
    implementation("androidx.constraintlayout:constraintlayout:2.1.0")
    implementation("androidx.coordinatorlayout:coordinatorlayout:1.1.0")
    implementation("androidx.core:core-ktx:1.6.0")
    implementation("androidx.fragment:fragment-ktx:$fragmentVersion")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.3.1")
    implementation("androidx.lifecycle:lifecycle-viewmodel-ktx:2.3.1")
    implementation("androidx.recyclerview:recyclerview:1.2.1")
    implementation("com.google.android.material:material:1.4.0")
    implementation("commons-validator:commons-validator:1.7")
    implementation("io.insert-koin:koin-core:$koinVersion")
    implementation("io.insert-koin:koin-core-ext:$koinVersion")
    implementation("io.insert-koin:koin-androidx-fragment:$koinVersion")
    implementation("io.insert-koin:koin-androidx-scope:$koinVersion")
    implementation("io.insert-koin:koin-androidx-viewmodel:$koinVersion")
    implementation("joda-time:joda-time:2.10.2")
    implementation("org.jetbrains.kotlin:kotlin-reflect:$kotlinVersion")
    implementation("org.jetbrains.kotlin:kotlin-stdlib:$kotlinVersion")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.5.1")

    /* Test dependencies */
    testImplementation("io.insert-koin:koin-test:$koinVersion")
    testImplementation("io.mockk:mockk:$mockkVersion")
    testImplementation("junit:junit:4.13")
    testImplementation("org.jetbrains.kotlin:kotlin-test:$kotlinVersion")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.5.1")

    /* UI test dependencies */
    debugImplementation("androidx.fragment:fragment-testing:$fragmentVersion")
    androidTestImplementation("androidx.test.espresso:espresso-core:$espressoVersion")
    androidTestImplementation("androidx.test.espresso:espresso-contrib:$espressoVersion")
    androidTestImplementation("androidx.test.ext:junit:1.1.3")
    androidTestImplementation("io.mockk:mockk-android:$mockkVersion")
    androidTestImplementation("io.insert-koin:koin-test:$koinVersion")
    androidTestImplementation("org.jetbrains.kotlin:kotlin-test:$kotlinVersion")
}
