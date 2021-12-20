plugins {
    id("org.owasp.dependencycheck") version "6.5.0.1" apply false
}

buildscript {
    val espressoVersion by extra { "3.3.0" }
    val fragmentVersion by extra { "1.3.2" }
    val koinVersion by extra { "2.2.2" }
    val kotlinVersion by extra { "1.4.31" }
    val mockkVersion by extra { "1.12.0" }

    repositories {
        google()
        mavenCentral()
        maven("https://plugins.gradle.org/m2/")
    }

    dependencies {
        classpath("com.android.tools.build:gradle:4.1.3")
        classpath("com.github.triplet.gradle:play-publisher:2.7.5")
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:$kotlinVersion")
        classpath("org.owasp:dependency-check-gradle:6.5.0.1")
    }
}

allprojects {
    apply(plugin = "org.owasp.dependencycheck")

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
