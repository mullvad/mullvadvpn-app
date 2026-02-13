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
    ":lib:common-compose",
    ":lib:common-test",
    ":lib:grpc",
    ":lib:endpoint",
    ":lib:feature:account:impl",
    ":lib:feature:addtime:impl",
    ":lib:feature:anticensorship:impl",
    ":lib:feature:apiaccess:impl",
    ":lib:feature:appinfo:impl",
    ":lib:feature:appearance:impl",
    ":lib:feature:autoconnect:impl",
    ":lib:feature:customlist:impl",
    ":lib:feature:daita:impl",
    ":lib:feature:filter:impl",
    ":lib:feature:home:impl",
    ":lib:feature:login:impl",
    ":lib:feature:managedevices:impl",
    ":lib:feature:multihop:impl",
    ":lib:feature:notification:impl",
    ":lib:feature:problemreport:impl",
    ":lib:feature:redeemvoucher:impl",
    ":lib:feature:serveripoverride:impl",
    ":lib:feature:settings:impl",
    ":lib:feature:splittunneling:impl",
    ":lib:feature:vpnsettings:impl",
    ":lib:map",
    ":lib:model",
    ":lib:navigation",
    ":lib:payment",
    ":lib:push-notification",
    ":lib:repository",
    ":lib:screen-test",
    ":lib:talpid",
    ":lib:tv",
    ":lib:ui:designsystem",
    ":lib:ui:component",
    ":lib:ui:icon",
    ":lib:ui:resource",
    ":lib:ui:tag",
    ":lib:ui:theme",
    ":lib:ui:util",
    ":lib:usecase",
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
