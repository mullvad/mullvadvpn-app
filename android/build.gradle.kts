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
    id("me.sigptr.rust-android") apply false

    alias(libs.plugins.detekt) apply true
    alias(libs.plugins.dependency.versions) apply true
    alias(libs.plugins.baselineprofile) apply false
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
        // https://github.com/gradle/gradle/issues/19228
        //noinspection UseTomlInstead
        val (aapt, aaptVersion) = with(libs.android.gradle.aapt.get()) { module to version }
        val agpVersion = libs.plugins.android.gradle.plugin.get().version.requiredVersion
        classpath("$aapt:$agpVersion-$aaptVersion:linux")
        classpath("$aapt:$agpVersion-$aaptVersion:osx")
        classpath("$aapt:$agpVersion-$aaptVersion:windows")

        // Protoc plugin
        classpath(libs.protobuf.protoc) {
            mapOf(
                    "linux-aarch_64" to "exe",
                    "linux-ppcle_64" to "exe",
                    "linux-s390_64" to "exe",
                    "linux-x86_32" to "exe",
                    "linux-x86_64" to "exe",
                    "osx-aarch_64" to "exe",
                    "osx-x86_64" to "exe",
                    "windows-x86_32" to "exe",
                    "windows-x86_64" to "exe",
                )
                .forEach { classifier, extension ->
                    artifact {
                        this.name = name
                        this.classifier = classifier
                        this.extension = extension
                    }
                }
        }

        // ProtoC gen grpc java plugin
        classpath(libs.grpc.protoc.gen.grpc.java) {
            mapOf(
                    "linux-aarch_64" to "exe",
                    "linux-ppcle_64" to "exe",
                    "linux-s390_64" to "exe",
                    "linux-x86_32" to "exe",
                    "linux-x86_64" to "exe",
                    "osx-aarch_64" to "exe",
                    "osx-x86_64" to "exe",
                    "windows-x86_32" to "exe",
                    "windows-x86_64" to "exe",
                )
                .forEach { classifier, extension ->
                    artifact {
                        this.name = name
                        this.classifier = classifier
                        this.extension = extension
                    }
                }
        }

        // Kotlin Native Prebuilt
        classpath(libs.kotlin.native.prebuilt) {
            mapOf(
                    "linux-x86_64" to "tar.gz",
                    "windows-x86_64" to "zip",
                    "macos-aarch64" to "tar.gz",
                    "macos-x86_64" to "tar.gz",
                )
                .forEach { (classifier, extension) ->
                    artifact {
                        this.name = name
                        this.classifier = classifier
                        this.type = extension
                    }
                }
        }
    }
}

detekt {
    val baselineFile = file("$rootDir/config/detekt-baseline.xml")
    val configFile = files("$rootDir/config/detekt.yml")
    val projectSource = file(projectDir)

    buildUponDefaultConfig = true
    allRules = false
    config.setFrom(configFile)
    source.setFrom(projectSource)
    parallel = true
    ignoreFailures = false
    autoCorrect = true
    baseline = baselineFile

    dependencies { detektPlugins(project(":test:detekt")) }
}

val detektExcludedPaths = listOf("**/build/**", "**/mullvad_daemon/management_interface/**")

tasks.withType<Detekt>().configureEach {
    dependsOn(":test:detekt:assemble")
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
