plugins {
    id(Dependencies.Plugin.dependencyCheckId) version Versions.Plugin.dependencyCheck apply false
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
    }
}

tasks.register("clean", Delete::class) {
    delete(rootProject.buildDir)
}
