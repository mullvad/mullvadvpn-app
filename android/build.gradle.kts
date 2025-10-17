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

    dependencies {
        detektPlugins(project(":test:detekt"))
    }
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
