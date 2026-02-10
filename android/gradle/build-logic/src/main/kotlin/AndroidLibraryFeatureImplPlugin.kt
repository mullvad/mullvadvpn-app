import com.android.build.api.dsl.LibraryExtension
import de.mannodermaus.gradle.plugins.junit5.dsl.AndroidJUnitPlatformExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies
import utilities.libs

class AndroidLibraryFeatureImplPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "mullvad.android-library")
            apply(plugin = "mullvad.unit-test")
            apply(plugin = "mullvad.android-library-instrumented-test")

            extensions.configure<LibraryExtension> {
                testOptions.animationsDisabled = true

                defaultConfig {
                    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
                }

                packaging {
                    resources {
                        pickFirsts +=
                            setOf(
                                // Fixes packaging error caused by: jetified-junit-*
                                "META-INF/LICENSE.md"
                            )
                    }
                }
            }

            extensions.configure<AndroidJUnitPlatformExtension> {
                instrumentationTests {
                    version.set(libs.findPlugin("junit5.android").get().get().version.strictVersion)
                    includeExtensions.set(true)
                }
            }

            dependencies {
                "implementation"(project(":lib:model"))
                "implementation"(project(":lib:common"))
                "implementation"(project(":lib:common-compose"))
                "implementation"(project(":lib:ui:theme"))
                "implementation"(project(":lib:ui:designsystem"))
                "implementation"(project(":lib:ui:component"))
                "implementation"(project(":lib:ui:resource"))
                "implementation"(project(":lib:ui:tag"))
                "implementation"(project(":lib:navigation"))
                "testImplementation"(project(":lib:common-test"))

                "androidTestImplementation"(libs.findLibrary("junit5.android.test.core").get())
                "androidTestImplementation"(
                    libs.findLibrary("junit5.android.test.extensions").get()
                )
                "androidTestImplementation"(libs.findLibrary("junit5.android.test.runner").get())
            }
        }
    }
}
