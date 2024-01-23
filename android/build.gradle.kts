import com.github.benmanes.gradle.versions.updates.DependencyUpdatesTask
import io.gitlab.arturbosch.detekt.Detekt
import io.gitlab.arturbosch.detekt.DetektCreateBaselineTask

plugins {
    id(Dependencies.Plugin.dependencyCheckId) version Versions.Plugin.dependencyCheck apply false
    id(Dependencies.Plugin.gradleVersionsId) version Versions.Plugin.gradleVersions
    id(Dependencies.Plugin.ktfmtId) version Versions.Plugin.ktfmt apply false
    id(Dependencies.Plugin.detektId) version Versions.Plugin.detekt
}

buildscript {
    repositories {
        google()
        mavenCentral()
        maven(Repositories.GradlePlugins)
        gradlePluginPortal()
    }

    dependencies {
        classpath(Dependencies.Plugin.android)
        classpath(Dependencies.Plugin.playPublisher)
        classpath(Dependencies.Plugin.kotlin)
        classpath(Dependencies.Plugin.dependencyCheck)

        // Required for Gradle metadata verification to work properly, see:
        // https://github.com/gradle/gradle/issues/19228
        classpath(Dependencies.Plugin.aaptLinux)
        classpath(Dependencies.Plugin.aaptOsx)
        classpath(Dependencies.Plugin.aaptWindows)
    }
}

val baselineFile = file("$rootDir/config/baseline.xml")
val configFile = files("$rootDir/config/detekt.yml")

val projectSource = file(projectDir)
val kotlinFiles = "**/*.kt"
val resourceFiles = "**/resources/**"
val buildFiles = "**/build/**"

detekt {
    buildUponDefaultConfig = true
    allRules = false
    config.setFrom(configFile)
    source.setFrom(projectSource)
    baseline = baselineFile
    parallel = true
    ignoreFailures = false
    autoCorrect = true
}

tasks.withType<Detekt>().configureEach {
    exclude(buildFiles)
}

// Kotlin DSL
tasks.withType<Detekt>().configureEach { jvmTarget = "1.8" }

tasks.withType<DetektCreateBaselineTask>().configureEach { jvmTarget = "1.8" }

allprojects {
    apply(plugin = Dependencies.Plugin.dependencyCheckId)

    repositories {
        google()
        mavenCentral()
    }

    configure<org.owasp.dependencycheck.gradle.extension.DependencyCheckExtension> {
        failBuildOnCVSS = 0F // All severity levels
        suppressionFile = "${rootProject.projectDir}/config/dependency-check-suppression.xml"
    }
}

tasks.withType<DependencyUpdatesTask> {
    gradleReleaseChannel = "current"
    rejectVersionIf { candidate.version.isNonStableVersion() }
}

tasks.register("clean", Delete::class) { delete(rootProject.buildDir) }
