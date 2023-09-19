import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties
import com.android.build.gradle.internal.tasks.factory.dependsOn
import java.io.FileInputStream
import java.util.*
import org.gradle.configurationcache.extensions.capitalized

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

val credentialsPath = "${rootProject.projectDir}/credentials"
val keystorePropertiesFile = file("$credentialsPath/keystore.properties")
val keystoreProperties = Properties()

if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    namespace = "net.mullvad.mullvadvpn"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        val localProperties = gradleLocalProperties(rootProject.projectDir)

        applicationId = "net.mullvad.mullvadvpn"
        minSdk = Versions.Android.minSdkVersion
        targetSdk = Versions.Android.targetSdkVersion
        versionCode = generateVersionCode(localProperties)
        versionName = generateVersionName(localProperties)
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        lint {
            lintConfig = file("${rootProject.projectDir}/config/lint.xml")
            baseline = file("lint-baseline.xml")
            abortOnError = true
            warningsAsErrors = true
        }
    }

    if (keystorePropertiesFile.exists()) {
        signingConfigs {
            create(SigningConfigs.RELEASE) {
                storeFile = file("$credentialsPath/app-keys.jks")
                storePassword = keystoreProperties.getProperty("storePassword")
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        getByName(BuildTypes.RELEASE) {
            signingConfig = signingConfigs.findByName(SigningConfigs.RELEASE)
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        create(BuildTypes.FDROID) {
            initWith(buildTypes.getByName(BuildTypes.RELEASE))
            signingConfig = null
            matchingFallbacks += BuildTypes.RELEASE
        }
        create(BuildTypes.LEAK_CANARY) {
            initWith(buildTypes.getByName(BuildTypes.DEBUG))
            applicationIdSuffix = ".leakcanary"
            matchingFallbacks += BuildTypes.DEBUG
        }
    }

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) {
            dimension = FlavorDimensions.BILLING
            isDefault = true
        }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            isDefault = true
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            applicationId = "net.mullvad.mullvadvpn.devmole"
        }
    }

    sourceSets {
        getByName("main") {
            val changelogDir =
                gradleLocalProperties(rootProject.projectDir)
                    .getOrDefault("OVERRIDE_CHANGELOG_DIR", defaultChangeLogAssetsDirectory)

            assets.srcDirs(extraAssetsDirectory, changelogDir)
            jniLibs.srcDirs(extraJniDirectory)
        }
    }

    buildFeatures { compose = true }

    composeOptions { kotlinCompilerExtensionVersion = Versions.kotlinCompilerExtensionVersion }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        allWarningsAsErrors = false
        jvmTarget = Versions.jvmTarget
        freeCompilerArgs =
            listOf(
                "-opt-in=kotlinx.coroutines.ExperimentalCoroutinesApi",
                "-opt-in=kotlinx.coroutines.ObsoleteCoroutinesApi",
                // Opt-in option for Koin annotation of KoinComponent.
                "-opt-in=kotlin.RequiresOptIn"
            )
    }

    tasks.withType<com.android.build.gradle.tasks.MergeSourceSetFolders> {
        dependsOn(getTasksByName("copyExtraAssets", true))
    }

    // Suppressing since we don't seem have much of an option than using this api. The impact should
    // also be limited to tests.
    @Suppress("UnstableApiUsage")
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

    packaging {
        jniLibs.useLegacyPackaging = true
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: androidx.compose.ui:ui-test-junit4
                    "META-INF/AL2.0",
                    "META-INF/LGPL2.1",
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md"
                )
        }
    }

    applicationVariants.configureEach {
        val alwaysShowChangelog =
            gradleLocalProperties(rootProject.projectDir).getProperty("ALWAYS_SHOW_CHANGELOG")
                ?: "false"

        buildConfigField("boolean", "ALWAYS_SHOW_CHANGELOG", alwaysShowChangelog)

        val enableInAppVersionNotifications =
            gradleLocalProperties(rootProject.projectDir)
                .getProperty("ENABLE_IN_APP_VERSION_NOTIFICATIONS")
                ?: "true"

        buildConfigField(
            "boolean",
            "ENABLE_IN_APP_VERSION_NOTIFICATIONS",
            enableInAppVersionNotifications
        )
    }

    applicationVariants.all {
        val artifactSuffix = buildString {
            productFlavors.getOrNull(0)?.name?.let { billingFlavorName ->
                if (billingFlavorName != Flavors.OSS) {
                    append(".$billingFlavorName")
                }
            }

            productFlavors.getOrNull(1)?.name?.let { infrastructureFlavorName ->
                if (infrastructureFlavorName != Flavors.PROD) {
                    append(".$infrastructureFlavorName")
                }
            }

            if (buildType.name != BuildTypes.RELEASE) {
                append(".${buildType.name}")
            }
        }

        val variantName = name
        val capitalizedVariantName = variantName.capitalized()
        val artifactName = "MullvadVPN-${versionName}${artifactSuffix}"

        tasks.register<Copy>("create${capitalizedVariantName}DistApk") {
            from(packageApplicationProvider)
            into("${rootDir.parent}/dist")
            include { it.name.endsWith(".apk") }
            rename { "$artifactName.apk" }
        }

        val createDistBundle =
            tasks.register<Copy>("create${capitalizedVariantName}DistBundle") {
                from("$buildDir/outputs/bundle/$variantName")
                into("${rootDir.parent}/dist")
                include { it.name.endsWith(".aab") }
                rename { "$artifactName.aab" }
            }

        createDistBundle.dependsOn("bundle$capitalizedVariantName")
    }

    project.tasks.preBuild.dependsOn("ensureJniDirectoryExist")
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.enable =
            variantBuilder.let { currentVariant ->
                val enabledVariants =
                    enabledVariantTriples.map { (billing, infra, buildType) ->
                        billing + infra.capitalized() + buildType.capitalized()
                    }
                enabledVariants.contains(currentVariant.name)
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

tasks.register("copyExtraAssets", Copy::class) {
    from("$repoRootPath/build")
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

afterEvaluate {
    tasks.withType(com.android.build.gradle.internal.lint.AndroidLintAnalysisTask::class.java) {
        mustRunAfter(tasks.getByName("copyExtraAssets"))
    }
}

play { serviceAccountCredentials.set(file("play-api-key.json")) }

configurations.all {
    resolutionStrategy {
        // Hold back emoji2 since newer versions require api level 34 which is not yet stable.
        force("androidx.emoji2:emoji2:1.3.0")
    }
}

dependencies {
    implementation(project(Dependencies.Mullvad.vpnService))
    implementation(project(Dependencies.Mullvad.tileService))

    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.endpointLib))
    implementation(project(Dependencies.Mullvad.ipcLib))
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.resourceLib))
    implementation(project(Dependencies.Mullvad.talpidLib))
    implementation(project(Dependencies.Mullvad.themeLib))
    implementation(project(Dependencies.Mullvad.paymentLib))

    // Play implementation
    playImplementation(project(Dependencies.Mullvad.billingLib))

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
    implementation(Dependencies.Compose.composeCollapsingToolbar)
    implementation(Dependencies.Compose.constrainLayout)
    implementation(Dependencies.Compose.foundation)
    implementation(Dependencies.Compose.viewModelLifecycle)
    implementation(Dependencies.Compose.material3)
    implementation(Dependencies.Compose.uiController)
    implementation(Dependencies.Compose.ui)
    implementation(Dependencies.Compose.uiUtil)
    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Koin.core)
    implementation(Dependencies.Koin.android)
    implementation(Dependencies.Koin.compose)
    implementation(Dependencies.Kotlin.reflect)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    // UI tooling
    implementation(Dependencies.Compose.uiToolingPreview)
    debugImplementation(Dependencies.Compose.uiTooling)

    // Leak canary
    leakCanaryImplementation(Dependencies.leakCanary)

    // Test dependencies
    testImplementation(project(Dependencies.Mullvad.commonTestLib))
    testImplementation(Dependencies.Koin.test)
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.junit)
    testImplementation(Dependencies.turbine)

    // UI test dependencies
    debugImplementation(Dependencies.AndroidX.fragmentTestning)
    // Fixes: https://github.com/android/android-test/issues/1589
    debugImplementation(Dependencies.AndroidX.testMonitor)
    debugImplementation(Dependencies.Compose.testManifest)
    androidTestImplementation(Dependencies.AndroidX.espressoContrib)
    androidTestImplementation(Dependencies.AndroidX.espressoCore)
    androidTestImplementation(Dependencies.Compose.junit)
    androidTestImplementation(Dependencies.Koin.test)
    androidTestImplementation(Dependencies.Kotlin.test)
    androidTestImplementation(Dependencies.MockK.android)
}
