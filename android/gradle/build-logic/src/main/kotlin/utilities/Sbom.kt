package utilities

import org.cyclonedx.Version
import org.cyclonedx.gradle.CyclonedxDirectTask
import org.cyclonedx.model.Component
import org.cyclonedx.model.ExternalReference
import org.gradle.api.Project
import org.gradle.kotlin.dsl.register

private val sbomVariants = listOf("ossProdRelease", "playProdRelease")

fun Project.registerSbomTasks(versionName: String) {
    pluginManager.apply("org.cyclonedx.bom")
    sbomVariants.forEach { variant ->
        val cap = variant.replaceFirstChar(Char::uppercase)
        tasks.register<CyclonedxDirectTask>("generate${cap}Sbom") {
            includeConfigs.set(listOf("${variant}RuntimeClasspath"))
            schemaVersion.set(Version.VERSION_15)
            includeBuildSystem.set(false)
            projectType.set(Component.Type.APPLICATION)
            componentName.set("mullvadvpn-app-android")
            componentVersion.set(versionName)
            // Set to enable forks to reproduce our SBOMs
            externalReferences.add(
                ExternalReference().apply {
                    type = ExternalReference.Type.VCS
                    url = "https://github.com/mullvad/mullvadvpn-app/"
                }
            )
            jsonOutput.set(layout.buildDirectory.file("reports/$variant.jvm.cdx.json"))
        }
    }
}
