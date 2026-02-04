import org.gradle.api.Plugin
import org.gradle.api.Project

class UtilitiesPlugin : Plugin<Project> {
    override fun apply(project: Project) {
        // Plugin intentionally empty - utilities are available via package import
    }
}
