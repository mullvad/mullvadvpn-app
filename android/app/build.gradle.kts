import com.android.build.api.artifact.SingleArtifact
import com.android.build.api.variant.BuildConfigField
import com.github.triplet.gradle.androidpublisher.ReleaseStatus
import org.gradle.internal.extensions.stdlib.capitalized
import utilities.BuildTypes
import utilities.FlavorDimensions
import utilities.Flavors
import utilities.Variant
import utilities.allPlayDebugReleaseVariants
import utilities.appVersionProvider
import utilities.baselineFilter
import utilities.fullReleaseTasks
import utilities.generateRemapArguments
import utilities.getBooleanProperty
import utilities.getStringListProperty
import utilities.isReleaseBuild
import utilities.leakCanaryImplementation
import utilities.matchesAny
import utilities.ossProdAnyBuildType
import utilities.playImplementation
import utilities.registerReleaseTask

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.application)
    alias(libs.plugins.play.publisher)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.compose)
    alias(libs.plugins.baselineprofile)
    alias(libs.plugins.mullvad.unit.test)
    alias(libs.plugins.rust.android)
    id("de.mannodermaus.android-junit5")
}

val repoRootPath = rootProject.projectDir.absoluteFile.parentFile.absolutePath
val relayListDirectory = file("$repoRootPath/dist-assets/relays/").absolutePath
val changelogAssetsDirectory = "$repoRootPath/android/src/main/play/release-notes/"
val rustJniLibsDir = layout.buildDirectory.dir("rustJniLibs/android").get()

val appVersion = appVersionProvider.get()

android {
    namespace = "net.mullvad.mullvadvpn"
    compileSdk = libs.versions.compile.sdk.major.get().toInt()
    compileSdkMinor = libs.versions.compile.sdk.minor.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()
    ndkVersion = libs.versions.ndk.get()

    defaultConfig {
        applicationId = "net.mullvad.mullvadvpn"
        minSdk = libs.versions.min.sdk.get().toInt()
        targetSdk = libs.versions.target.sdk.get().toInt()
        versionCode = appVersion.code
        versionName = appVersion.name
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
        register("playDevmoleRelease") { enabled = appVersion.isAlpha }
        register("playStagemoleRelease") { enabled = appVersion.isAlpha }
        register("playProdRelease") {
            enabled = !appVersion.isDev
            releaseStatus.set(ReleaseStatus.DRAFT)
            track.set(
                when {
                    appVersion.isStable -> "production"
                    appVersion.isBeta -> "beta"
                    else -> "internal"
                }
            )
        }
    }

    androidResources {
        @Suppress("UnstableApiUsage")
        // Due to a bug in the Android platform we need to disable this as the auto-generated local
        // config causes a crash on some versions of android.
        // See: https://issuetracker.google.com/issues/399131926#comment29
        // Restoring this behavior when the issue has been resolved is tracked in: DROID-2163
        generateLocaleConfig = false
    }

    buildTypes {
        getByName(BuildTypes.RELEASE) {
            signingConfig = null
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
        getByName(BuildTypes.DEBUG) { isPseudoLocalesEnabled = true }
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
            buildConfigField("String", "API_ENDPOINT", "\"\"")
            buildConfigField("String", "API_IP", "\"\"")
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            applicationId = "net.mullvad.mullvadvpn.devmole"
            buildConfigField("String", "API_ENDPOINT", "\"api-app.devmole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.4\"")
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            applicationId = "net.mullvad.mullvadvpn.stagemole"
            buildConfigField("String", "API_ENDPOINT", "\"api-app.stagemole.eu\"")
            buildConfigField("String", "API_IP", "\"185.217.116.132\"")
        }
    }

    sourceSets {
        getByName("main") {
            assets.directories.add(relayListDirectory)
            assets.directories.add(changelogAssetsDirectory)
        }
        // Workaround to include all instrumented tests in app module. Without this we'd have to
        // create an APK for each submodule and pass each on for testing with the orchestrator.
        getByName("androidTest") {
            val instrumentedTests =
                rootProject.subprojects
                    .mapNotNull { subProject ->
                        subProject.file("src/androidTest/kotlin").takeIf { it.exists() }
                    }
                    .map { it.absolutePath }
            kotlin.directories.addAll(instrumentedTests)
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    testOptions {
        unitTests.all { test ->
            test.testLogging {
                test.outputs.upToDateWhen { false }
                events("passed", "skipped", "failed", "standardOut", "standardError")
                showCauses = true
                showExceptions = true
                showStandardStreams = true
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
}

androidComponents {
    onVariants { variant ->
        val mainSources = variant.sources.getByName("main")
        mainSources.addStaticSourceDirectory(relayListDirectory)
        mainSources.addStaticSourceDirectory(changelogAssetsDirectory)
    }

    onVariants {
        it.buildConfigFields!!.put(
            "ENABLE_IN_APP_VERSION_NOTIFICATIONS",
            BuildConfigField(
                "boolean",
                getBooleanProperty("mullvad.app.config.inAppVersionNotifications.enable"),
                "Show in-app version notifications",
            ),
        )
        val shouldRequireBundleRelayFile = isReleaseBuild() && !appVersion.isDev
        it.buildConfigFields!!.put(
            "REQUIRE_BUNDLED_RELAY_FILE",
            BuildConfigField(
                "boolean",
                shouldRequireBundleRelayFile.toString(),
                "Whether to require a bundled relay list or not.",
            ),
        )
    }
    onVariants {
        val productFlavors = it.productFlavors.toMap()
        val buildType = it.buildType

        val artifactSuffix = buildString {
            productFlavors[FlavorDimensions.BILLING]?.let { billingFlavorName ->
                if (billingFlavorName != Flavors.OSS) {
                    append(".$billingFlavorName")
                }
            }

            productFlavors[FlavorDimensions.INFRASTRUCTURE]?.let { infrastructureFlavorName ->
                if (infrastructureFlavorName != Flavors.PROD) {
                    append(".$infrastructureFlavorName")
                }
            }

            if (buildType != BuildTypes.RELEASE) {
                append(".${buildType}")
            }
        }

        val variantName = it.name
        val capitalizedVariantName = variantName.capitalized()
        val artifactName = "MullvadVPN-${appVersion.name}${artifactSuffix}"

        tasks.register<Copy>("create${capitalizedVariantName}DistApk") {
            from(it.artifacts.get(SingleArtifact.APK))
            into("${rootDir.parent}/dist")
            include { it.name.endsWith(".apk") }
            rename { "$artifactName.apk" }
        }

        tasks.register<Copy>("create${capitalizedVariantName}DistBundle") {
            from(it.artifacts.get(SingleArtifact.BUNDLE))
            into("${rootDir.parent}/dist")
            include { it.name.endsWith(".aab") }
            rename { "$artifactName.aab" }
        }

        tasks.findByPath("generate${capitalizedVariantName}BaselineProfile")?.let {
            val baselineFile = "baseline-prof.txt"
            val sourceDir = "${rootProject.projectDir}/app/src"
            val fromDir = "$sourceDir/$variantName/generated/baselineProfiles"
            val toDir = "$sourceDir/main"
            val fromFile = file("$fromDir/$baselineFile")
            val toFile = file("$toDir/$baselineFile")
            it.doLast { fromFile.renameTo(toFile) }
        }
    }
}

// Ensure that we have all the JNI libs before merging them.
tasks
    .matching { it.name.matches(Regex("merge.*JniLibFolders")) }
    .configureEach {
        // This is required for the merge task to run every time the .so files are updated.
        // See this comment for more information:
        // https://github.com/mozilla/rust-android-gradle/issues/118#issuecomment-1569407058
        inputs.dir(rustJniLibsDir)
        dependsOn("cargoBuild")
    }

kotlin {
    compilerOptions {
        allWarningsAsErrors = true
        freeCompilerArgs =
            listOf(
                // Opt-in option for Koin annotation of KoinComponent.
                "-opt-in=kotlin.RequiresOptIn",
                "-XXLanguage:+WhenGuards",
            )
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
    val enableApiOverride = !isReleaseBuild || appVersion.isDev || appVersion.isAlpha
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
    targetDirectory = "$repoRootPath/target"
    features {
        val enabledFeatures =
            buildList {
                    if (enableApiOverride) {
                        add("api-override")
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

    if (getBooleanProperty("mullvad.app.build.replaceRustPathPrefix")) {
        environmentalOverrides["RUSTFLAGS"] = generateRemapArguments()
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
    val versionCode = project.android.defaultConfig.versionCode
    val versionName = project.android.defaultConfig.versionName
    doLast {
        println("versionCode=$versionCode")
        println("versionName=$versionName")
    }
}

tasks.register("debug") { dependsOn("assembleOssProdDebug") }

tasks.register("debugPlay") { dependsOn("assemblePlayProdDebug") }

registerReleaseTask(
    "fdroidRelease",
    appVersion,
    listOf("createOssProdReleaseDistApk"),
    skipClean = true,
    skipDirtyCheck = true,
)

registerReleaseTask("fullRelease", appVersion, fullReleaseTasks(appVersion))

play {
    System.getenv("PLAY_CREDENTIALS_PATH")?.let { serviceAccountCredentials.set(file(it)) }
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
    implementation(project(":lib:common-compose"))
    implementation(projects.lib.grpc)
    implementation(projects.lib.endpoint)
    implementation(projects.lib.feature.account.impl)
    implementation(projects.lib.feature.addtime.impl)
    implementation(projects.lib.feature.anticensorship.impl)
    implementation(projects.lib.feature.anticensorship.api)
    implementation(projects.lib.feature.apiaccess.impl)
    implementation(projects.lib.feature.apiaccess.api)
    implementation(projects.lib.feature.appicon.impl)
    implementation(projects.lib.feature.appinfo.impl)
    implementation(projects.lib.feature.appinfo.api)
    implementation(projects.lib.feature.applisting.impl)
    implementation(projects.lib.feature.applisting.api)
    implementation(projects.lib.feature.appearance.impl)
    implementation(projects.lib.feature.autoconnect.impl)
    implementation(projects.lib.feature.customlist.impl)
    implementation(projects.lib.feature.customlist.api)
    implementation(projects.lib.feature.daita.impl)
    implementation(projects.lib.feature.deleteaccount.impl)
    implementation(projects.lib.feature.daita.api)
    implementation(projects.lib.feature.filter.impl)
    implementation(projects.lib.feature.home.impl)
    implementation(projects.lib.feature.home.api)
    implementation(projects.lib.feature.language.impl)
    implementation(projects.lib.feature.location.impl)
    implementation(projects.lib.feature.location.api)
    implementation(projects.lib.feature.login.impl)
    implementation(projects.lib.feature.login.api)
    implementation(projects.lib.feature.managedevices.impl)
    implementation(projects.lib.feature.multihop.impl)
    implementation(projects.lib.feature.notification.impl)
    implementation(projects.lib.feature.problemreport.impl)
    implementation(projects.lib.feature.redeemvoucher.impl)
    implementation(projects.lib.feature.serveripoverride.impl)
    implementation(projects.lib.feature.serveripoverride.api)
    implementation(projects.lib.feature.settings.impl)
    implementation(projects.lib.feature.settings.api)
    implementation(projects.lib.feature.splittunneling.impl)
    implementation(projects.lib.feature.vpnsettings.impl)
    implementation(projects.lib.feature.vpnsettings.api)
    implementation(projects.lib.map)
    implementation(projects.lib.model)
    implementation(projects.lib.pushNotification)
    implementation(projects.lib.navigation)
    implementation(projects.lib.payment)
    implementation(projects.lib.repository)
    implementation(projects.lib.talpid)
    implementation(projects.lib.tv)
    implementation(projects.lib.ui.designsystem)
    implementation(projects.lib.ui.component)
    implementation(projects.lib.ui.icon)
    implementation(projects.lib.ui.resource)
    implementation(projects.lib.ui.tag)
    implementation(projects.lib.ui.theme)
    implementation(projects.lib.ui.util)
    implementation(projects.lib.usecase)
    implementation(libs.androidx.profileinstaller)
    implementation(libs.androidx.navigation3.ui)

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
    implementation(libs.accompanist.permissions)
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
    implementation(libs.androidx.work.runtime.ktx)
    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.arrow.resilience)
    implementation(libs.compose.constrainlayout)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.icons.extended)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.util)

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
    androidTestImplementation(libs.androidx.espresso)
    androidTestImplementation(projects.lib.screenTest)
}
