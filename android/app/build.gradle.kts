import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties
import com.android.build.gradle.internal.lint.LintModelWriterTask
import com.android.build.gradle.internal.tasks.factory.dependsOn
import com.android.build.gradle.tasks.MergeSourceSetFolders
import java.io.FileInputStream
import java.util.Properties
import org.gradle.configurationcache.extensions.capitalized

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.play.publisher)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.kotlin.ksp)
    alias(libs.plugins.compose)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

val repoRootPath = rootProject.projectDir.absoluteFile.parentFile.absolutePath
val extraAssetsDirectory = "${project.buildDir}/extraAssets"
val relayListPath = "$extraAssetsDirectory/relays.json"
val maybenotMachinesDirectory = "$extraAssetsDirectory/maybenot_machines"
val defaultChangelogAssetsDirectory = "$repoRootPath/android/src/main/play/release-notes/"
val extraJniDirectory = "${project.buildDir}/extraJni"

val credentialsPath = "${rootProject.projectDir}/credentials"
val keystorePropertiesFile = file("$credentialsPath/keystore.properties")
val keystoreProperties = Properties()

if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    namespace = "net.mullvad.mullvadvpn"
    compileSdk = Versions.compileSdkVersion

    defaultConfig {
        val localProperties = gradleLocalProperties(rootProject.projectDir, providers)

        applicationId = "net.mullvad.mullvadvpn"
        minSdk = Versions.minSdkVersion
        targetSdk = Versions.targetSdkVersion
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

    androidResources {
        @Suppress("UnstableApiUsage")
        generateLocaleConfig = true
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
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            applicationId = "net.mullvad.mullvadvpn.stagemole"
        }
    }

    sourceSets {
        getByName("main") {
            val changelogDir =
                gradleLocalProperties(rootProject.projectDir, providers)
                    .getOrDefault("OVERRIDE_CHANGELOG_DIR", defaultChangelogAssetsDirectory)

            assets.srcDirs(extraAssetsDirectory, changelogDir)
            jniLibs.srcDirs(extraJniDirectory)
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

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
                    "META-INF/LICENSE-notice.md",
                    "META-INF/io.netty.versions.properties",
                    "META-INF/INDEX.LIST"
                )
        }
    }

    applicationVariants.configureEach {
        val alwaysShowChangelog =
            gradleLocalProperties(rootProject.projectDir, providers)
                .getProperty("ALWAYS_SHOW_CHANGELOG") ?: "false"

        buildConfigField("boolean", "ALWAYS_SHOW_CHANGELOG", alwaysShowChangelog)

        val enableInAppVersionNotifications =
            gradleLocalProperties(rootProject.projectDir, providers)
                .getProperty("ENABLE_IN_APP_VERSION_NOTIFICATIONS") ?: "true"

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

        // Ensure all relevant assemble tasks depend on our ensure tasks.
        tasks.get("assemble$capitalizedVariantName").apply {
            dependsOn(tasks.get("ensureRelayListExist"))
            dependsOn(tasks.get("ensureMaybenotMachinesExist"))
            dependsOn(tasks.get("ensureJniDirectoryExist"))
            dependsOn(tasks.get("ensureValidVersionCode"))
        }
    }
}

junitPlatform {
    instrumentationTests {
        version.set(Versions.junit5Android)
        includeExtensions.set(true)
    }
}

composeCompiler { enableStrongSkippingMode = true }

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.enable =
            variantBuilder.let { currentVariant ->
                val enabledVariants =
                    enabledAppVariantTriples.map { (billing, infra, buildType) ->
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

tasks.register("ensureRelayListExist") {
    doLast {
        if (!file(relayListPath).exists()) {
            throw GradleException("Missing relay list: $relayListPath")
        }
    }
}

tasks.register("ensureMaybenotMachinesExist") {
    doLast {
        if (!file(maybenotMachinesDirectory).exists()) {
            throw GradleException("Missing maybenot machines: $maybenotMachinesDirectory")
        }
    }
}

tasks.register("ensureJniDirectoryExist") {
    doLast {
        if (!file(extraJniDirectory).exists()) {
            throw GradleException("Missing JNI directory: $extraJniDirectory")
        }
    }
}

// This is a safety net to avoid generating too big version codes, since that could potentially be
// hard and inconvenient to recover from.
tasks.register("ensureValidVersionCode") {
    doLast {
        val versionCode = project.android.defaultConfig.versionCode!!
        if (versionCode >= MAX_ALLOWED_VERSION_CODE) {
            throw GradleException("Bad version code: $versionCode")
        }
    }
}

tasks.create("printVersion") {
    doLast {
        println("versionCode=${project.android.defaultConfig.versionCode}")
        println("versionName=${project.android.defaultConfig.versionName}")
    }
}

play { serviceAccountCredentials.set(file("play-api-key.json")) }

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.endpoint)
    implementation(projects.lib.intentProvider)
    implementation(projects.lib.map)
    implementation(projects.lib.model)
    implementation(projects.lib.payment)
    implementation(projects.lib.resource)
    implementation(projects.lib.shared)
    implementation(projects.lib.talpid)
    implementation(projects.tile)
    implementation(projects.lib.theme)
    implementation(projects.service)

    // Play implementation
    playImplementation(projects.lib.billing)

    implementation(libs.commons.validator)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.ktx)
    implementation(libs.androidx.coresplashscreen)
    implementation(libs.androidx.lifecycle.runtime)
    implementation(libs.androidx.lifecycle.viewmodel)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.arrow.resilience)
    implementation(libs.compose.constrainlayout)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.util)
    implementation(libs.compose.destinations)
    ksp(libs.compose.destinations.ksp)

    implementation(libs.jodatime)
    implementation(libs.kermit)
    implementation(libs.koin)
    implementation(libs.koin.android)
    implementation(libs.koin.compose)
    implementation(libs.kotlin.reflect)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)

    // UI tooling
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)

    // Leak canary
    leakCanaryImplementation(libs.leakCanary)

    // Needed for createComposeExtension() and createAndroidComposeExtension()
    debugImplementation(libs.compose.ui.test.manifest)
    testImplementation(projects.lib.commonTest)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.turbine)
    testImplementation(Dependencies.junitJupiterApi)
    testRuntimeOnly(Dependencies.junitJupiterEngine)
    testImplementation(Dependencies.junitJupiterParams)

    // UI test dependencies
    debugImplementation(libs.compose.ui.test.manifest)
    androidTestImplementation(libs.koin.test)
    androidTestImplementation(libs.kotlin.test)
    androidTestImplementation(libs.mockk.android)
    androidTestImplementation(Dependencies.junitJupiterApi)
    androidTestImplementation(Dependencies.junit5AndroidTestCompose)
}
