import com.github.benmanes.gradle.versions.updates.DependencyUpdatesTask
import utilities.PreBuildTask
import utilities.appVersionProvider
import utilities.isNonStableVersion
import utilities.isReleaseBuild

plugins {
    alias(libs.plugins.mullvad.utilities)
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.test) apply false
    alias(libs.plugins.ktfmt) apply false
    alias(libs.plugins.compose) apply false
    alias(libs.plugins.play.publisher) apply false
    alias(libs.plugins.kotlin.ksp) apply false
    alias(libs.plugins.kotlin.parcelize) apply false
    alias(libs.plugins.protobuf.core) apply false
    alias(libs.plugins.rust.android) apply false
    alias(libs.plugins.wire) apply false

    alias(libs.plugins.detekt) apply true
    alias(libs.plugins.dependency.versions) apply true
    alias(libs.plugins.baselineprofile) apply false
}

buildscript {
    dependencies {
        //noinspection UseTomlInstead
        // Dependency class paths are required for Gradle metadata verification to work properly,
        // see:
        // https://github.com/gradle/gradle/issues/19228

        if (gradle.startParameter.writeDependencyVerifications.isNotEmpty()) {
            println("Writing dependency verification file, adding platform specific classpaths")
            val aapt = libs.android.gradle.aapt.get()
            val aaptVersion =
                "${libs.versions.android.gradle.plugin.get()}-${libs.versions.android.gradle.aapt.get()}"
            classpath("$aapt:$aaptVersion:linux")
            classpath("$aapt:$aaptVersion:osx")
            classpath("$aapt:$aaptVersion:windows")

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
        }
    }

    // These depencies are added by the Wire plugin, but they are not needed for our build so we
    // exclude them.
    // Unfortunately, this is not possible to do using libs.version.toml
    // https://github.com/gradle/gradle/issues/26367#issuecomment-2120830998
    configurations.classpath {
        exclude(group = "it.krzeminski", module = "snakeyaml-engine-kmp-jvm")
        exclude(group = "it.krzeminski", module = "snakeyaml-engine-kmp")
        exclude(group = "io.outfoxx", module = "swiftpoet")
        exclude(group = "com.squareup.wire", module = "wire-swift-generator")
    }
}

allprojects {
    apply(plugin = rootProject.libs.plugins.ktfmt.get().pluginId)

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

// The preflight configuration is done at the project root level to ensure
// it runs before any other build tasks. This is a known limitation:
// https://github.com/gradle/gradle/issues/29064
//
// The substringAfterLast split is used in order to catch both the plain
// and fully qualified task names ("fdroidRelease" and ":app:fdroidRelease").
val preflightSkipDirtyCheck =
    gradle.startParameter.taskNames.any { it.substringAfterLast(':') == "fdroidRelease" }
val releasePreflight =
    tasks.register<PreBuildTask>("releasePreflight") {
        this.skipDirtyCheck.set(preflightSkipDirtyCheck)
        this.versionName.set(appVersionProvider.map { it.name })
    }

if (isReleaseBuild()) {
    allprojects {
        tasks.configureEach { if (name != "releasePreflight") dependsOn(releasePreflight) }
    }
}
