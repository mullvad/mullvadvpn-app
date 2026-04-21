import com.android.build.api.dsl.LibraryExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.internal.Actions.with
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.assign
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension
import utilities.libs

class AndroidLibraryPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "com.android.library")
            apply(plugin = "mullvad.kotlin-toolchain")

            extensions.configure<LibraryExtension> {
                compileSdk = libs.findVersion("compile-sdk").get().toString().toInt()
                buildToolsVersion = libs.findVersion("build-tools").get().toString()

                defaultConfig { minSdk = libs.findVersion("min-sdk").get().toString().toInt() }

                lint {
                    lintConfig = file("${project.rootDir}/config/lint.xml")
                    baseline =
                        file("${rootProject.projectDir.absolutePath}/config/lint-baseline.xml")
                    abortOnError = true
                    warningsAsErrors = true
                }
                buildFeatures { buildConfig = true }
            }
            extensions.configure<KotlinAndroidProjectExtension> {
                compilerOptions { allWarningsAsErrors = true }
            }
        }
    }
}
