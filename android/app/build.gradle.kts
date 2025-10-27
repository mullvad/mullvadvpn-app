import com.android.build.gradle.internal.tasks.factory.dependsOn
import com.github.triplet.gradle.androidpublisher.ReleaseStatus
import java.io.FileInputStream
import java.util.Properties
import org.gradle.internal.extensions.stdlib.capitalized
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.play.publisher)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.compose)
    alias(libs.plugins.junit5.android)
    alias(libs.plugins.baselineprofile)
    id("me.sigptr.rust-android")
}

val repoRootPath = rootProject.projectDir.absoluteFile.parentFile.absolutePath
val relayListDirectory = file("$repoRootPath/dist-assets/relays/").absolutePath
val changelogAssetsDirectory = "$repoRootPath/android/src/main/play/release-notes/"
val rustJniLibsDir = layout.buildDirectory.dir("rustJniLibs/android").get()

val credentialsPath = "${rootProject.projectDir}/credentials"
val keystorePropertiesFile = file("$credentialsPath/keystore.properties")
val keystoreProperties = Properties()

if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    namespace = "net.mullvad.mullvadvpn"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()
    ndkVersion = libs.versions.ndk.get()

    defaultConfig {
        applicationId = "net.mullvad.mullvadvpn"
        minSdk = libs.versions.min.sdk.get().toInt()
        targetSdk = libs.versions.target.sdk.get().toInt()
        versionCode = generateVersionCode()
        versionName = generateVersionName()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        lint {
            lintConfig = file("${rootProject.projectDir}/config/lint.xml")
            baseline = file("${rootProject.projectDir}/config/lint-baseline.xml")
            abortOnError = true
            checkAllWarnings = true
            warningsAsErrors = true
            checkDependencies = true
        }
    }

    playConfigs {
        register("playDevmoleRelease") { enabled = true }
        register("playStagemoleRelease") { enabled = true }
        register("playProdRelease") { enabled = true }
    }

    androidResources {
        @Suppress("UnstableApiUsage")
        // Due to a bug in the Android platform we need to disable this as the auto-generated local
        // config causes a crash on some versions of android.
        // See: https://issuetracker.google.com/issues/399131926#comment29
        // Restoring this behavior when the issue has been resolved is tracked in: DROID-2163
        generateLocaleConfig = false
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
                "proguard-rules.pro",
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
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            applicationId = "net.mullvad.mullvadvpn.stagemole"
        }
    }

    sourceSets {
        getByName("main") { assets.srcDirs(relayListDirectory, changelogAssetsDirectory) }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin {
        compilerOptions {
            jvmTarget = JvmTarget.fromTarget(libs.versions.jvm.target.get())
            allWarningsAsErrors = true
            freeCompilerArgs =
                listOf(
                    // Opt-in option for Koin annotation of KoinComponent.
                    "-opt-in=kotlin.RequiresOptIn",
                    "-XXLanguage:+WhenGuards",
                )
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

    packaging {
        if (getBooleanProperty("mullvad.app.build.keepDebugSymbols")) {
            jniLibs.keepDebugSymbols.add("**/*.so")
        }
        jniLibs.useLegacyPackaging = true
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: androidx.compose.ui:ui-test-junit4
                    "META-INF/AL2.0",
                    "META-INF/LGPL2.1",
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md",
                    "META-INF/io.netty.versions.properties",
                    "META-INF/INDEX.LIST",
                )
        }
    }

    applicationVariants.configureEach {
        buildConfigField(
            "boolean",
            "ENABLE_IN_APP_VERSION_NOTIFICATIONS",
            getBooleanProperty("mullvad.app.config.inAppVersionNotifications.enable").toString(),
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
        val capitalizedVariantName = variantName.toString().capitalized()
        val artifactName = "MullvadVPN-${versionName}${artifactSuffix}"

        tasks.register<Copy>("create${capitalizedVariantName}DistApk") {
            from(packageApplicationProvider)
            into("${rootDir.parent}/dist")
            include { it.name.endsWith(".apk") }
            rename { "$artifactName.apk" }
        }

        val createDistBundle =
            tasks.register<Copy>("create${capitalizedVariantName}DistBundle") {
                from("${layout.buildDirectory.get()}/outputs/bundle/$variantName")
                into("${rootDir.parent}/dist")
                include { it.name.endsWith(".aab") }
                rename { "$artifactName.aab" }
            }

        createDistBundle.dependsOn("bundle$capitalizedVariantName")

        // Ensure that we have all the JNI libs before merging them.
        tasks["merge${capitalizedVariantName}JniLibFolders"].apply {
            // This is required for the merge task to run every time the .so files are updated.
            // See this comment for more information:
            // https://github.com/mozilla/rust-android-gradle/issues/118#issuecomment-1569407058
            inputs.dir(rustJniLibsDir)
            dependsOn("cargoBuild")
        }
    }
}

junitPlatform {
    instrumentationTests {
        version.set(libs.versions.junit5.android.asProvider())
        includeExtensions.set(true)
    }
}

cargo {
    val isReleaseBuild = isReleaseBuild()
    val generateDebugSymbolsForReleaseBuilds =
        getBooleanProperty("mullvad.app.build.cargo.generateDebugSymbolsForReleaseBuilds")
    val enableBoringTun = getBooleanProperty("mullvad.app.build.boringtun.enable")
    val enableApiOverride = !isReleaseBuild || isDevBuild() || isAlphaBuild()
    module = repoRootPath
    libname = "mullvad-jni"
    // All available targets:
    // https://github.com/mozilla/rust-android-gradle/tree/master?tab=readme-ov-file#targets
    targets = getStringListProperty("mullvad.app.build.cargo.targets")
    profile =
        if (isReleaseBuild) {
            if (generateDebugSymbolsForReleaseBuilds) "release-debuginfo" else "release"
        } else {
            "debug"
        }
    prebuiltToolchains = true
    targetDirectory = "$repoRootPath/target"
    features {
        val enabledFeatures =
            buildList {
                    if (enableApiOverride) {
                        add("api-override")
                    }
                    if (enableBoringTun) {
                        add("boringtun")
                    }
                }
                .toTypedArray()

        @Suppress("SpreadOperator") defaultAnd(*enabledFeatures)
    }
    targetIncludes = arrayOf("libmullvad_jni.so")
    extraCargoBuildArguments = buildList {
        add("--package=mullvad-jni")
        add("--locked")
    }
    exec = { spec, _ ->
        println("Executing Cargo: ${spec.commandLine.joinToString(" ")}")

        if (getBooleanProperty("mullvad.app.build.replaceRustPathPrefix"))
            spec.environment("RUSTFLAGS", generateRemapArguments())
    }
}

tasks.register<Exec>("cargoClean") {
    workingDir = File(repoRootPath)
    commandLine("cargo", "clean")
}

if (getBooleanProperty("mullvad.app.build.cargo.cleanBuild")) {
    tasks["clean"].dependsOn("cargoClean")
}

baselineProfile { warnings { disabledVariants = false } }

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.enable =
            Variant(variantBuilder.buildType, variantBuilder.productFlavors)
                .matchesAny(allPlayDebugReleaseVariants, ossProdAnyBuildType, baselineFilter)
    }
}

tasks.register("printVersion") {
    doLast {
        println("versionCode=${project.android.defaultConfig.versionCode}")
        println("versionName=${project.android.defaultConfig.versionName}")
    }
}

play {
    serviceAccountCredentials.set(file("$credentialsPath/play-api-key.json"))
    // Disable for all flavors by default. Only specific flavors should be enabled using
    // PlayConfigs.
    enabled = false
    // This property refers to the Publishing API (not git).
    commit = true
    defaultToAppBundles = true
    track = "internal"
    releaseStatus = ReleaseStatus.COMPLETED
    userFraction = 1.0
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.endpoint)
    implementation(projects.lib.map)
    implementation(projects.lib.model)
    implementation(projects.lib.payment)
    implementation(projects.lib.resource)
    implementation(projects.lib.repository)
    implementation(projects.lib.talpid)
    implementation(projects.lib.tv)
    implementation(projects.lib.ui.designsystem)
    implementation(projects.lib.ui.component)
    implementation(projects.lib.ui.tag)
    implementation(projects.tile)
    implementation(projects.lib.theme)
    implementation(projects.service)
    implementation(libs.androidx.profileinstaller)

    // Baseline profile
    baselineProfile(projects.test.baselineprofile)

    // Play implementation
    playImplementation(projects.lib.billing)

    // This dependency can be replaced when minimum SDK is 29 or higher.
    // It can then be replaced with InetAddress.isNumericAddress
    implementation(libs.commons.validator) {
        // This dependency has a known vulnerability
        // https://osv.dev/vulnerability/GHSA-wxr5-93ph-8wr9
        // It is not used so let's exclude it.
        // Unfortunately, this is not possible to do using libs.version.toml
        // https://github.com/gradle/gradle/issues/26367#issuecomment-2120830998
        exclude("commons-beanutils", "commons-beanutils")
    }
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.datastore)
    implementation(libs.androidx.coresplashscreen)
    implementation(libs.androidx.credentials) {
        // This dependency adds a lot of unused permissions to the app.
        // It is not used so let's exclude it.
        // Unfortunately, this is not possible to do using libs.version.toml
        // https://github.com/gradle/gradle/issues/26367#issuecomment-2120830998
        exclude("androidx.biometric", "biometric")
    }
    implementation(libs.androidx.ktx)
    implementation(libs.androidx.lifecycle.runtime)
    implementation(libs.androidx.lifecycle.viewmodel)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.tv)
    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.arrow.resilience)
    implementation(libs.compose.constrainlayout)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.icons.extended)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.util)
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)

    implementation(libs.accompanist.drawablepainter)

    implementation(libs.kermit)
    implementation(libs.koin)
    implementation(libs.koin.android)
    implementation(libs.koin.compose)
    implementation(libs.kotlin.reflect)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.protobuf.kotlin.lite)

    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)

    // Leak canary
    leakCanaryImplementation(libs.leakCanary)

    testImplementation(projects.lib.commonTest)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.turbine)
    testImplementation(libs.junit.jupiter.api)
    testRuntimeOnly(libs.junit.jupiter.engine)
    testImplementation(libs.junit.jupiter.params)

    // HACK:
    // Not used by app module, but otherwise an older version pre 1.8.0 will be used at runtime for
    // the e2e tests. This causes the deserialization to fail because of a missing function that was
    // introduced in 1.8.0.
    implementation(libs.kotlinx.serialization.json)

    // UI test dependencies

    // Needed for createComposeExtension() and createAndroidComposeExtension()
    debugImplementation(libs.compose.ui.test.manifest)
    androidTestImplementation(libs.koin.test)
    androidTestImplementation(libs.kotlin.test)
    androidTestImplementation(libs.mockk.android)
    androidTestImplementation(libs.turbine)
    androidTestImplementation(libs.junit.jupiter.api)
    androidTestImplementation(libs.junit5.android.test.compose)
}
