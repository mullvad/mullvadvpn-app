import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties
import org.gradle.internal.extensions.stdlib.capitalized
import java.util.Properties

plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlinx.serialization)

    id(Dependencies.junit5AndroidPluginId) version Versions.junit5Plugin
}

android {
    namespace = "net.mullvad.mullvadvpn.test.e2e"
    compileSdk = Versions.compileSdkVersion
    buildToolsVersion = Versions.buildToolsVersion

    defaultConfig {
        minSdk = Versions.minSdkVersion
        testApplicationId = "net.mullvad.mullvadvpn.test.e2e"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        testInstrumentationRunnerArguments["runnerBuilder"] =
            "de.mannodermaus.junit5.AndroidJUnit5Builder"
        targetProjectPath = ":app"

        fun Properties.addRequiredPropertyAsBuildConfigField(name: String) {
            val value =
                System.getenv(name)
                    ?: getProperty(name)
                    ?: throw GradleException("Missing property: $name")

            buildConfigField(type = "String", name = name, value = "\"$value\"")
        }

        Properties().apply {
            load(project.file("e2e.properties").inputStream())
            addRequiredPropertyAsBuildConfigField("API_VERSION")
            addRequiredPropertyAsBuildConfigField("ENABLE_HIGHLY_RATE_LIMITED_TESTS")
            addRequiredPropertyAsBuildConfigField("ENABLE_ACCESS_TO_LOCAL_API_TESTS")
            addRequiredPropertyAsBuildConfigField("TRAFFIC_GENERATION_IP_ADDRESS")
            addRequiredPropertyAsBuildConfigField("PACKET_CAPTURE_API_HOST")
        }

        fun MutableMap<String, String>.addOptionalPropertyAsArgument(name: String) {
            val value =
                rootProject.properties.getOrDefault(name, null) as? String
                    ?: gradleLocalProperties(rootProject.projectDir, providers).getProperty(name)

            if (value != null) {
                put(name, value)
            }
        }

        testInstrumentationRunnerArguments +=
            mutableMapOf<String, String>().apply {
                put("clearPackageData", "true")
                addOptionalPropertyAsArgument("valid_test_account_number")
                addOptionalPropertyAsArgument("invalid_test_account_number")
            }
    }

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField(
                type = "String",
                name = "INFRASTRUCTURE_BASE_DOMAIN",
                value = "\"mullvad.net\"",
            )
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField(
                type = "String",
                name = "INFRASTRUCTURE_BASE_DOMAIN",
                value = "\"stagemole.eu\"",
            )
        }
    }

    testOptions { execution = "ANDROIDX_TEST_ORCHESTRATOR" }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
        allWarningsAsErrors = true
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    packaging {
        resources {
            pickFirsts +=
                setOf(
                    // Fixes packaging error caused by: jetified-junit-*
                    "META-INF/LICENSE.md",
                    "META-INF/LICENSE-notice.md",
                )
        }
    }
    buildFeatures { buildConfig = true }
}

junitPlatform {
    instrumentationTests {
        version.set(Versions.junit5Android)
        includeExtensions.set(true)
    }
}

androidComponents {
    beforeVariants { variantBuilder ->
        variantBuilder.enable =
            variantBuilder.let { currentVariant ->
                val enabledVariants =
                    enabledE2eVariantTriples.map { (billing, infra, buildType) ->
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
    suppressionFile = "$projectDir/../test-suppression.xml"
}

dependencies {
    implementation(projects.test.common)
    implementation(projects.lib.endpoint)
    implementation(libs.androidx.test.core)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(libs.androidx.test.monitor)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.android.volley)
    implementation(libs.kermit)
    implementation(Dependencies.junitJupiterApi)
    implementation(Dependencies.junit5AndroidTestExtensions)
    implementation(Dependencies.junit5AndroidTestRunner)
    implementation(libs.kotlin.stdlib)
    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.cio)
    implementation(libs.ktor.serialization.kotlinx.json)
    implementation(libs.ktor.client.content.negotiation)
    implementation(libs.jodatime)

    androidTestUtil(libs.androidx.test.orchestrator)

    // Needed or else the app crashes when launched
    implementation(Dependencies.junit5AndroidTestCompose)
    implementation(libs.compose.material3)

    // Need these for forcing later versions of dependencies
    implementation(libs.compose.ui)
    implementation(libs.androidx.activity.compose)
}
