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
}

gradlePlugin {
    plugins {
        register("utilities") {
            id = "mullvad.utilities"
            implementationClass = "MullvadUtilitiesPlugin"
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
}
