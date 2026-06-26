import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies
import org.jetbrains.kotlin.gradle.ExperimentalKotlinGradlePluginApi
import org.jetbrains.kotlin.powerassert.gradle.PowerAssertGradleExtension
import utilities.libs

class UnitTestPlugin : Plugin<Project> {
    @OptIn(ExperimentalKotlinGradlePluginApi::class)
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "mullvad.kotlin-toolchain")
            apply(plugin = "de.infix.testBalloon")
            apply(plugin = "org.jetbrains.kotlin.plugin.power-assert")

            configure<PowerAssertGradleExtension> {
                functions.set(
                    listOf(
                        "kotlin.assert",
                        "kotlin.test.assertTrue",
                        "kotlin.test.assertEquals",
                        "kotlin.test.assertNull",
                    )
                )
                includedSourceSets.set(
                    listOf(
                        "debugAndroidTest",
                        "debugUnitTest",
                        "test",
                        "testDebug",
                        "testRelease",
                        "testOssProdDebug",
                    )
                )
            }

            dependencies {
                "testImplementation"(libs.findLibrary("kotlinx.coroutines.test").get())
                "testImplementation"(libs.findLibrary("junit").get())
                "testImplementation"(libs.findLibrary("mockk").get())
                "testImplementation"(libs.findLibrary("turbine").get())
                "testImplementation"(libs.findLibrary("kotlin.test").get())
                "testImplementation"(libs.findLibrary("test.balloon.framework").get())
                "testImplementation"(libs.findLibrary("kotlin.power.assert.runtime").get())
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
