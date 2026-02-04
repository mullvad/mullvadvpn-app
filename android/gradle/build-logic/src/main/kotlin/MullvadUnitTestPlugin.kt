import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.UnknownTaskException
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.dependencies
import utilities.libs

class MullvadUnitTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "de.mannodermaus.android-junit5")
            dependencies {
                "testImplementation"(project(":lib:common-test"))
                "testImplementation"(libs.findLibrary("kotlinx.coroutines.test").get())
                "testImplementation"(libs.findLibrary("mockk").get())
                "testImplementation"(libs.findLibrary("junit-jupiter-api").get())
                "testImplementation"(libs.findLibrary("junit.jupiter.params").get())
                "testRuntimeOnly"(libs.findLibrary("junit-jupiter-engine").get())
                "testImplementation"(libs.findLibrary("turbine").get())
                "testImplementation"(libs.findLibrary("kotlin.test").get())
                "testImplementation"(libs.findLibrary("junit5.android.test.core").get())
                "testImplementation"(libs.findLibrary("junit5.android.test.extensions").get())
            }

            tasks.register("testAllUnitTests") {
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
}
