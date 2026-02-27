import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.configure
import org.jetbrains.kotlin.gradle.dsl.KotlinProjectExtension
import org.jetbrains.kotlin.gradle.plugin.KotlinBasePluginWrapper
import utilities.libs

class KotlinToolchainPlugin : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            // Use 'withType' to ensure this only runs if a Kotlin plugin is applied
            // 'KotlinBasePluginWrapper' covers Kotlin JVM, Android, and Multiplatform
            plugins.withType(KotlinBasePluginWrapper::class.java) {
                extensions.configure<KotlinProjectExtension> {
                    jvmToolchain(libs.findVersion("jvm-toolchain").get().toString().toInt())
                }
            }
        }
    }
}
