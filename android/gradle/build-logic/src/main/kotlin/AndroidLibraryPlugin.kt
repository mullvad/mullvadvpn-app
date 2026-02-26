import com.android.build.gradle.LibraryExtension
import org.gradle.api.JavaVersion
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.plugins.JavaPluginExtension
import org.gradle.internal.Actions.with
import org.gradle.jvm.toolchain.JavaLanguageVersion
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.assign
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.getByType
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension
import utilities.libs

class AndroidLibraryPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "com.android.library")
            apply(plugin = "org.jetbrains.kotlin.android")
            apply(plugin = "mullvad.kotlin-toolchain")

            extensions.configure<LibraryExtension> {
                compileSdk = libs.findVersion("compile-sdk").get().toString().toInt()
                buildToolsVersion = libs.findVersion("build-tools").get().toString()

                defaultConfig { minSdk = libs.findVersion("min-sdk").get().toString().toInt() }

                lint {
                    lintConfig = file("${project.rootDir}/config/lint.xml")
                    abortOnError = true
                    warningsAsErrors = true
                }
                buildFeatures { buildConfig = true }
            }
            extensions.configure<KotlinAndroidProjectExtension> {
                compilerOptions {
                    allWarningsAsErrors = true
                }
            }
        }
    }
}
