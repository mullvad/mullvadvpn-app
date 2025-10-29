plugins {
    `kotlin-dsl`
    alias(libs.plugins.ktfmt) apply true
    alias(libs.plugins.detekt) apply true
}

kotlin { jvmToolchain(17) }

// Should be the same as ktfmt config in project root build.gradle.kts
ktfmt {
    kotlinLangStyle()
    maxWidth.set(100)
    removeUnusedImports.set(true)
}
