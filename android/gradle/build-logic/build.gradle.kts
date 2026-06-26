plugins {
    `kotlin-dsl`
    alias(libs.plugins.ktfmt)
    alias(libs.plugins.kotlin.power.assert)
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
    implementation(libs.kotlin.power.assert.plugin)
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
        register("unit-test-balloon") {
            id = "mullvad.unit-test-balloon"
            implementationClass = "UnitTestBalloonPlugin"
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
}
