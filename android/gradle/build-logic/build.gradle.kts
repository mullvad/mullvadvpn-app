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
        register("utilities") {
            id = "mullvad.utilities"
            implementationClass = "MullvadUtilitiesPlugin"
        }
    }
    plugins {
        register("unit-test") {
            id = "mullvad.unit-test"
            implementationClass = "MullvadUnitTestPlugin"
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
