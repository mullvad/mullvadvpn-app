import com.android.build.api.dsl.LibraryExtension
import de.mannodermaus.gradle.plugins.junit5.dsl.AndroidJUnitPlatformExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies
import utilities.libs

class AndroidLibraryInstrumentedTestPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "mullvad.android-library")
            apply(plugin = "de.mannodermaus.android-junit5")

            extensions.configure<LibraryExtension> {
                testOptions.animationsDisabled = true

                defaultConfig {
                    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
                }
            }

            extensions.configure<AndroidJUnitPlatformExtension> {
                instrumentationTests {
                    version.set(libs.findPlugin("junit5.android").get().get().version.strictVersion)
                    includeExtensions.set(true)
                }
            }

            dependencies {
                "androidTestImplementation"(libs.findLibrary("junit5.android.test.core").get())
                "androidTestImplementation"(
                    libs.findLibrary("junit5.android.test.extensions").get()
                )
                "androidTestImplementation"(libs.findLibrary("junit5.android.test.runner").get())
            }
        }
    }
}
