import org.gradle.api.Plugin
import org.gradle.api.Project

class MullvadTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        target.tasks.register("testAllUnitTests") {
            val testTask =
                target.tasks.findByName("testDebugUnitTest")
                    // Modules with flavors will not have normal test tasks so we test with
                    // ossProdDebug
                    ?: target.tasks.getByName("testOssProdDebugUnitTest")
            // This is to make sure that all unit tests are always executed
            testTask.outputs.upToDateWhen { false }
            dependsOn(testTask)
        }
    }
}
