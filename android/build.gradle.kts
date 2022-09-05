import com.github.benmanes.gradle.versions.updates.DependencyUpdatesTask

plugins {
    id(Dependencies.Plugin.dependencyCheckId) version Versions.Plugin.dependencyCheck apply false
    id(Dependencies.Plugin.gradleVersionsId) version Versions.Plugin.gradleVersions
}

buildscript {
    repositories {
        google()
        mavenCentral()
        maven(Repositories.GradlePlugins)
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
    resolutionStrategy {
        componentSelection {
            all {
                if (candidate.version.isNonStableVersion()) {
                    reject("Non-stable version.")
                }
            }
        }
    }
}

tasks.register("clean", Delete::class) {
    delete(rootProject.buildDir)
}
