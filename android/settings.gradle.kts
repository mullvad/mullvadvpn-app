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

include(":app")

include(
    ":lib:billing",
    ":lib:common",
    ":lib:common-compose",
    ":lib:common-test",
    ":lib:grpc",
    ":lib:endpoint",
    ":lib:feature:account:impl",
    ":lib:feature:account:api",
    ":lib:feature:addtime:impl",
    ":lib:feature:addtime:api",
    ":lib:feature:anticensorship:impl",
    ":lib:feature:anticensorship:api",
    ":lib:feature:apiaccess:impl",
    ":lib:feature:apiaccess:api",
    ":lib:feature:appicon:impl",
    ":lib:feature:appicon:api",
    ":lib:feature:appinfo:impl",
    ":lib:feature:appinfo:api",
    ":lib:feature:appearance:impl",
    ":lib:feature:appearance:api",
    ":lib:feature:autoconnect:impl",
    ":lib:feature:autoconnect:api",
    ":lib:feature:customlist:impl",
    ":lib:feature:customlist:api",
    ":lib:feature:daita:impl",
    ":lib:feature:daita:api",
    ":lib:feature:deleteaccount:impl",
    ":lib:feature:deleteaccount:api",
    ":lib:feature:filter:impl",
    ":lib:feature:filter:api",
    ":lib:feature:home:impl",
    ":lib:feature:home:api",
    ":lib:feature:language:impl",
    ":lib:feature:language:api",
    ":lib:feature:location:impl",
    ":lib:feature:location:api",
    ":lib:feature:login:impl",
    ":lib:feature:login:api",
    ":lib:feature:managedevices:impl",
    ":lib:feature:managedevices:api",
    ":lib:feature:multihop:impl",
    ":lib:feature:multihop:api",
    ":lib:feature:notification:impl",
    ":lib:feature:notification:api",
    ":lib:feature:problemreport:impl",
    ":lib:feature:problemreport:api",
    ":lib:feature:redeemvoucher:impl",
    ":lib:feature:redeemvoucher:api",
    ":lib:feature:serveripoverride:impl",
    ":lib:feature:serveripoverride:api",
    ":lib:feature:settings:impl",
    ":lib:feature:settings:api",
    ":lib:feature:splittunneling:impl",
    ":lib:feature:splittunneling:api",
    ":lib:feature:vpnsettings:impl",
    ":lib:feature:vpnsettings:api",
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
