import com.android.build.api.dsl.LibraryExtension
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
                                "META-INF/LICENSE.md",
                            )
                    }
                }
            }
//
//            junitPlatform {
//                instrumentationTests {
//                    version.set(libs.versions.junit5.android.asProvider())
//                    includeExtensions.set(true)
//                }
//            }


            dependencies {
                "implementation"(project(":lib:model"))
                "implementation"(project(":lib:common"))
                "implementation"(project(":lib:ui:theme"))
                "implementation"(project(":lib:ui:designsystem"))
                "implementation"(project(":lib:ui:component"))
                "implementation"(project(":lib:ui:resource"))
                "implementation"(project(":lib:ui:tag"))
                "implementation"(project(":lib:navigation"))
                "testImplementation"(project(":lib:common-test"))

                "testImplementation"(libs.findLibrary("kotlinx.coroutines.test").get())
                "testImplementation"(libs.findLibrary("mockk").get())
                "testImplementation"(libs.findLibrary("junit-jupiter-api").get())
                "testRuntimeOnly"(libs.findLibrary("junit-jupiter-engine").get())
                "testImplementation"(libs.findLibrary("turbine").get())
                "testImplementation"(libs.findLibrary("kotlin.test").get())
            }
        }
    }
}
