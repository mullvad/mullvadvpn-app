import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.tasks.testing.Test
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.dependencies
import utilities.libs

class UnitTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "mullvad.kotlin-toolchain")
            apply(plugin = "de.infix.testBalloon")
            apply(plugin = "org.jetbrains.kotlin.plugin.power-assert")

            dependencies {
                "testImplementation"(libs.findLibrary("kotlinx.coroutines.test").get())
                "testImplementation"(libs.findLibrary("junit").get())
                "testImplementation"(libs.findLibrary("mockk").get())
                "testImplementation"(libs.findLibrary("turbine").get())
                "testImplementation"(libs.findLibrary("kotlin.test").get())
                "testImplementation"(libs.findLibrary("test.balloon.framework").get())
            }

            tasks.register("testAllUnitTests") {
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
}
