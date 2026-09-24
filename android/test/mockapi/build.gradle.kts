import utilities.FlavorDimensions
import utilities.Flavors

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.test)
}

android {
    namespace = "net.mullvad.mullvadvpn.test.mockapi"
    compileSdk = libs.versions.compile.sdk.major.get().toInt()
    compileSdkMinor = libs.versions.compile.sdk.minor.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig {
        minSdk = libs.versions.min.sdk.get().toInt()
        testApplicationId = "net.mullvad.mullvadvpn.test.mockapi"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        // Required to run mock api tests locally
        testInstrumentationRunnerArguments["runnerBuilder"] =
            "de.mannodermaus.junit5.AndroidJUnit5Builder"
        targetProjectPath = ":app"

        missingDimensionStrategy(FlavorDimensions.BILLING, Flavors.OSS)
        missingDimensionStrategy(FlavorDimensions.INFRASTRUCTURE, Flavors.PROD)

        testInstrumentationRunnerArguments.putAll(mapOf("clearPackageData" to "true"))
    }

    flavorDimensions += FlavorDimensions.BILLING

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
    }

    testOptions { execution = "ANDROIDX_TEST_ORCHESTRATOR" }

    kotlin { compilerOptions { allWarningsAsErrors = true } }

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
                    // Fixes packaging error caused by: io.netty:netty
                    "META-INF/INDEX.LIST",
                    "META-INF/io.netty.versions.properties",
                    "META-INF/license/LICENSE.jbzip2.txt",
                    "META-INF/license/LICENSE.webbit.txt",
                    "META-INF/license/LICENSE.snappy.txt",
                    "META-INF/license/LICENSE.protobuf.txt",
                    "META-INF/license/LICENSE.jsr166y.txt",
                    "META-INF/license/NOTICE.harmony.txt",
                    "META-INF/license/LICENSE.mvn-wrapper.txt",
                    "META-INF/license/LICENSE.quiche.txt",
                    "META-INF/license/LICENSE.nghttp2-hpack.txt",
                    "META-INF/license/LICENSE.base64.txt",
                    "META-INF/license/LICENSE.commons-lang.txt",
                    "META-INF/license/LICENSE.jzlib.txt",
                    "META-INF/license/LICENSE.slf4j.txt",
                    "META-INF/license/LICENSE.aalto-xml.txt",
                    "META-INF/license/LICENSE.dnsinfo.txt",
                    "META-INF/license/LICENSE.jctools.txt",
                    "META-INF/license/LICENSE.zstd-jni.txt",
                    "META-INF/license/LICENSE.log4j.txt",
                    "META-INF/native-image/io.netty/netty-codec-native-quic/native-image.properties",
                    "META-INF/license/LICENSE.libdivsufsort.txt",
                    "META-INF/license/LICENSE.compress-lzf.txt",
                    "META-INF/license/LICENSE.jboss-marshalling.txt",
                    "META-INF/license/LICENSE.commons-logging.txt",
                    "META-INF/license/LICENSE.hyper-hpack.txt",
                    "META-INF/native-image/io.netty/netty-codec-native-quic/resource-config.json",
                    "META-INF/license/LICENSE.hpack.txt",
                    "META-INF/native-image/io.netty/netty-codec-native-quic/jni-config.json",
                    "META-INF/license/LICENSE.harmony.txt",
                    "META-INF/license/LICENSE.bouncycastle.txt",
                    "META-INF/license/LICENSE.lzma-java.txt",
                    "META-INF/native-image/io.netty/netty-codec-native-quic/reflect-config.json",
                    "META-INF/license/LICENSE.jfastlz.txt",
                    "META-INF/license/LICENSE.brotli4j.txt",
                    "META-INF/license/LICENSE.caliper.txt",
                    "META-INF/license/LICENSE.lz4.txt",
                    "META-INF/license/LICENSE.boringssl.txt",
                )
        }
    }
}

dependencies {
    implementation(projects.lib.endpoint)
    implementation(projects.test.common)
    implementation(projects.lib.ui.tag)
    implementation(projects.lib.model)

    implementation(libs.androidx.test.core)
    // Fixes: https://github.com/android/android-test/issues/1589
    implementation(libs.androidx.test.monitor)
    implementation(libs.androidx.test.runner)
    implementation(libs.androidx.test.rules)
    implementation(libs.androidx.test.uiautomator)
    implementation(libs.kermit)
    implementation(libs.junit.jupiter.api)
    implementation(libs.junit5.android.test.extensions)
    implementation(libs.junit5.android.test.runner)
    implementation(libs.kotlin.stdlib)
    implementation(libs.ktor.server.core)
    implementation(libs.ktor.server.engine.netty)

    androidTestUtil(libs.androidx.test.orchestrator)

    // Needed or else the app crashes when launched
    implementation(libs.junit5.android.test.compose)
    implementation(libs.compose.material3)

    // Need these for forcing later versions of dependencies
    implementation(libs.compose.ui)
    implementation(libs.androidx.activity.compose)
}
