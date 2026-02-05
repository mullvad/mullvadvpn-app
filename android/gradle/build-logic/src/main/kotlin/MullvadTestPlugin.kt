import org.gradle.api.Plugin
import org.gradle.api.Project

class MullvadTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.tasks.register("testAllUnitTests") {
            dependsOn(try {
                target.tasks.getByName("testDebugUnitTest")
            } catch (e: Exception) {
                // Modules with flavors will not have normal test task so we test with ossProdDebug
                target.tasks.getByName("testOssProdDebugUnitTest")
            })
        }
    }
}
