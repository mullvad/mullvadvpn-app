import dev.detekt.gradle.Detekt
import dev.detekt.gradle.DetektCreateBaselineTask
import dev.detekt.gradle.extensions.DetektExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies
import org.gradle.kotlin.dsl.withType

class DetektConfigPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "dev.detekt")

            extensions.configure<DetektExtension> {
                buildUponDefaultConfig.set(true)
                allRules.set(false)
                config.setFrom(rootProject.files("config/detekt.yml"))
                baseline.set(rootProject.file("config/detekt-baseline.xml"))
                parallel.set(true)
                ignoreFailures.set(false)
                autoCorrect.set(true)
            }

            dependencies { add("detektPlugins", project(":test:detekt")) }

            val detektExcludedPaths =
                listOf("**/build/**", "**/mullvad_daemon/management_interface/**", ".gradle/**")

            // Ensures the custom rules jar is built before any Detekt task executes
            tasks.withType<Detekt>().configureEach {
                dependsOn(":test:detekt:assemble")
                exclude(detektExcludedPaths)
            }

            tasks.withType<DetektCreateBaselineTask>().configureEach {
                // Ignore generated files from the build directory, e.g files created by ksp.
                exclude(detektExcludedPaths)
            }
        }
    }
}
