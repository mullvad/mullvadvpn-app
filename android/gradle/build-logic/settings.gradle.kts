dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal() {
            content { includeGroup("org.cyclonedx") }
        }
    }
    versionCatalogs { create("libs") { from(files("../libs.versions.toml")) } }
}

rootProject.name = "build-logic"
