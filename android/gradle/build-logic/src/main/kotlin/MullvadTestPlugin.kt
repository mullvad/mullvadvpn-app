import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.UnknownTaskException

class MullvadTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.tasks.register("testAllUnitTests") {
            dependsOn(
                try {
                    target.tasks.getByName("testDebugUnitTest")
                } catch (e: UnknownTaskException) {
                    // Modules with flavors will not have normal test tasks so we test with
                    // ossProdDebug
                    target.tasks.getByName("testOssProdDebugUnitTest")
                }
            )
        }
    }
}
