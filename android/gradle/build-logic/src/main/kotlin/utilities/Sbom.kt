package utilities

import org.cyclonedx.Version
import org.cyclonedx.gradle.CyclonedxDirectTask
import org.gradle.api.Project
import org.gradle.kotlin.dsl.register

private val sbomVariants = listOf("ossProdRelease", "playProdRelease")

// Emits a per-variant JVM/Android CycloneDX SBOM into build/reports.
fun Project.registerSbomTasks(versionName: String) {
    sbomVariants.forEach { variant ->
        val cap = variant.replaceFirstChar(Char::uppercase)
        tasks.register<CyclonedxDirectTask>("generate${cap}Sbom") {
            includeConfigs.set(listOf("${variant}RuntimeClasspath"))
            schemaVersion.set(Version.VERSION_15)
            componentName.set("mullvad-vpn-android")
            componentVersion.set(versionName)
            jsonOutput.set(layout.buildDirectory.file("reports/$variant.jvm.sbom.cdx.json"))
        }
    }
}
