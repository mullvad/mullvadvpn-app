import com.android.build.api.dsl.CommonExtension
import com.android.build.api.dsl.LibraryExtension
import org.gradle.api.JavaVersion
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.internal.Actions.with
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.assign
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension
import utilities.libs

class AndroidLibraryPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "com.android.library")
            extensions.configure<LibraryExtension> {
                compileSdk = libs.findVersion("compile-sdk").get().toString().toInt()
                buildToolsVersion = libs.findVersion("build-tools").get().toString()

                defaultConfig { minSdk = libs.findVersion("min-sdk").get().toString().toInt() }

                compileOptions {
                    sourceCompatibility = JavaVersion.VERSION_17
                    targetCompatibility = JavaVersion.VERSION_17
                }

                lint {
                    lintConfig = file("${project.rootDir}/config/lint.xml")
                    abortOnError = true
                    warningsAsErrors = true
                }
                buildFeatures { buildConfig = true }
            }
            extensions.configure<KotlinAndroidProjectExtension> {
                compilerOptions {
                    jvmTarget =
                        JvmTarget.fromTarget(libs.findVersion("jvm-target").get().toString())
                    allWarningsAsErrors = true
                }
            }
        }
    }
}
