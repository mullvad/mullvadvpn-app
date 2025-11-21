pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal() {
            content {
                // Exclude gRPC artifacts - they're only available in Maven Central
                excludeGroup("io.grpc")
            }
        }
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

includeBuild("rust-android-gradle-plugin")
includeBuild("gradle/build-logic")

enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

rootProject.name = "MullvadVPN"

include(
    ":app",
    ":service",
    ":tile"
)
include(
    ":lib:billing",
    ":lib:common",
    ":lib:common-test",
    ":lib:daemon-grpc",
    ":lib:endpoint",
    ":lib:map",
    ":lib:model",
    ":lib:payment",
    ":lib:resource",
    ":lib:repository",
    ":lib:talpid",
    ":lib:theme",
    ":lib:tv",
    ":lib:ui:designsystem",
    ":lib:ui:component",
    ":lib:ui:tag",
    ":lib:ui:util"
)
include(
    ":test",
    ":test:arch",
    ":test:common",
    ":test:e2e",
    ":test:mockapi",
    ":test:detekt",
    ":test:baselineprofile",
)
