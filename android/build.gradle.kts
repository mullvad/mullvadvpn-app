import com.github.benmanes.gradle.versions.updates.DependencyUpdatesTask
import io.gitlab.arturbosch.detekt.Detekt
import io.gitlab.arturbosch.detekt.DetektCreateBaselineTask

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.test) apply false
    alias(libs.plugins.ktfmt) apply false
    alias(libs.plugins.compose) apply false
    alias(libs.plugins.play.publisher) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.ksp) apply false
    alias(libs.plugins.kotlin.parcelize) apply false
    alias(libs.plugins.protobuf.core) apply false
    alias(libs.plugins.rust.android.gradle) apply false

    alias(libs.plugins.detekt) apply true
    alias(libs.plugins.dependency.versions) apply true
}

buildscript {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    dependencies {
        // Dependency class paths are required for Gradle metadata verification to work properly,
        // see:
        // https://github.com/gradle/gradle/issues/19228s
        //noinspection UseTomlInstead
        val aapt = libs.android.gradle.aapt.get().toString()
        val aaptVersion = libs.versions.android.gradle.aapt.get()
        val agpVersion = libs.versions.android.gradle.plugin.get()
        classpath("$aapt:$agpVersion-$aaptVersion:linux")
        classpath("$aapt:$agpVersion-$aaptVersion:osx")
        classpath("$aapt:$agpVersion-$aaptVersion:windows")

        // Protoc plugin
        val protoc = libs.plugins.protobuf.protoc.get().toString()
        classpath("$protoc:linux-aarch_64@exe")
        classpath("$protoc:linux-ppcle_64@exe")
        classpath("$protoc:linux-s390_64@exe")
        classpath("$protoc:linux-x86_32@exe")
        classpath("$protoc:linux-x86_64@exe")
        classpath("$protoc:osx-aarch_64@exe")
        classpath("$protoc:osx-x86_64@exe")
        classpath("$protoc:windows-x86_32@exe")
        classpath("$protoc:windows-x86_64@exe")

        // ProtoC gen grpc java plugin
        val protocJava = libs.plugins.grpc.protoc.gen.grpc.java.get().toString()
        classpath("$protocJava:linux-aarch_64@exe")
        classpath("$protocJava:linux-ppcle_64@exe")
        classpath("$protocJava:linux-s390_64@exe")
        classpath("$protocJava:linux-x86_32@exe")
        classpath("$protocJava:linux-x86_64@exe")
        classpath("$protocJava:osx-aarch_64@exe")
        classpath("$protocJava:osx-x86_64@exe")
        classpath("$protocJava:windows-x86_32@exe")
        classpath("$protocJava:windows-x86_64@exe")

        // Kotlin Native Prebuilt
        val prebuilt = libs.kotlin.native.prebuilt.get().toString()
        classpath("$prebuilt:windows-x86_64@zip")
        classpath("$prebuilt:linux-x86_64@tar.gz")
        classpath("$prebuilt:macos-aarch64@tar.gz")
        classpath("$prebuilt:macos-x86_64@tar.gz")

        classpath("org.mozilla.rust-android-gradle:plugin:${libs.versions.rust.android.gradle}")
    }
}

val configFile = files("$rootDir/config/detekt.yml")

val projectSource = file(projectDir)
val detektExcludedPaths = listOf("**/build/**", "**/mullvad_daemon/management_interface/**")

detekt {
    buildUponDefaultConfig = true
    allRules = false
    config.setFrom(configFile)
    source.setFrom(projectSource)
    parallel = true
    ignoreFailures = false
    autoCorrect = true
}

tasks.withType<Detekt>().configureEach {
    // Ignore generated files from the build directory, e.g files created by ksp.
    exclude(detektExcludedPaths)
}

tasks.withType<DetektCreateBaselineTask>().configureEach {
    // Ignore generated files from the build directory, e.g files created by ksp.
    exclude(detektExcludedPaths)
}

allprojects {
    apply(plugin = rootProject.libs.plugins.ktfmt.get().pluginId)

    repositories {
        google()
        mavenCentral()
    }

    // Should be the same as ktfmt config in buildSrc/build.gradle.kts
    configure<com.ncorti.ktfmt.gradle.KtfmtExtension> {
        kotlinLangStyle()
        maxWidth.set(100)
        removeUnusedImports.set(true)
    }
}

tasks.withType<DependencyUpdatesTask> {
    gradleReleaseChannel = "current"
    rejectVersionIf { candidate.version.isNonStableVersion() }
}

tasks.register("clean", Delete::class) { delete(rootProject.layout.buildDirectory) }
