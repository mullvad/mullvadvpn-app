plugins {
    `kotlin-dsl`
    alias(libs.plugins.ktfmt)
}

ktfmt {
    kotlinLangStyle()
    maxWidth.set(100)
    removeUnusedImports.set(true)
}

dependencies {
    implementation(libs.android.gradle.plugin)
    implementation(libs.kotlin.gradle.plugin)
    implementation(libs.android.gradle.junit5)
}

gradlePlugin {
    plugins {
        register("kotlin-toolchain") {
            id = "mullvad.kotlin-toolchain"
            implementationClass = "KotlinToolchainPlugin"
        }
    }
    plugins {
        register("utilities") {
            id = "mullvad.utilities"
            implementationClass = "UtilitiesPlugin"
        }
    }
    plugins {
        register("unit-test") {
            id = "mullvad.unit-test"
            implementationClass = "UnitTestPlugin"
        }
    }
    plugins {
        register("android-library") {
            id = "mullvad.android-library"
            implementationClass = "AndroidLibraryPlugin"
        }
    }
    plugins {
        register("android-library-feature-impl") {
            id = "mullvad.android-library-feature-impl"
            implementationClass = "AndroidLibraryFeatureImplPlugin"
        }
    }
    plugins {
        register("android-library-feature-api") {
            id = "mullvad.android-library-feature-api"
            implementationClass = "AndroidLibraryFeatureApiPlugin"
        }
    }
    plugins {
        register("android-library-compose") {
            id = "mullvad.android-library-compose"
            implementationClass = "AndroidLibraryComposePlugin"
        }
    }
    plugins {
        register("android-library-instrumented-test") {
            id = "mullvad.android-library-instrumented-test"
            implementationClass = "AndroidLibraryInstrumentedTestPlugin"
        }
    }
    plugins {
        register("wireProtoPatcher") {
            id = "mullvad.wire-proto-patcher"
            implementationClass = "WireProtoPatcherPlugin"
        }
    }
}

configurations
    .matching { it.name == "kotlinAbiValidationCompatClasspath" }
    .configureEach {
        resolutionStrategy {
            eachDependency {
                if (requested.group == "org.jetbrains.kotlin") {
                    // We want to force the project's Kotlin version here because otherwise the
                    // version wil be the latest published version within the KGP-declared version
                    // range [2.4.0-Beta2, 2.5.0).
                    useVersion(libs.versions.kotlin.asProvider().get())
                    because("Set the Kotlin ABI classpath to the project's Kotlin version.")
                }
            }
        }
    }
