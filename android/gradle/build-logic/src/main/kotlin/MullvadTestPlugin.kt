import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.UnknownTaskException

class MullvadTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.tasks.register("testAllUnitTests") {
            dependsOn(
                try {
                    target.tasks.getByName("testDebugUnitTest").apply {
                        this@apply.outputs.upToDateWhen { false }
                    }
                } catch (e: UnknownTaskException) {
                    // Modules with flavors will not have normal test tasks so we test with
                    // ossProdDebug
                    target.tasks.getByName("testOssProdDebugUnitTest").apply {
                        this@apply.outputs.upToDateWhen { false }
                    }
                }
            )
            outputs.upToDateWhen { false }
        }
    }
}
