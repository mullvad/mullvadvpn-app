plugins {
    `kotlin-dsl`
    alias(libs.plugins.ktfmt) apply true
    alias(libs.plugins.detekt) apply true
}

repositories { maven("https://plugins.gradle.org/m2/") }

kotlin { jvmToolchain(17) }

ktfmt {
    kotlinLangStyle()
    maxWidth.set(100)
    removeUnusedImports.set(true)
}
