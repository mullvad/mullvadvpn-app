import com.android.build.api.dsl.LibraryExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.dependencies

class AndroidLibraryFeatureImplPlugin: Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            apply(plugin = "mullvad.android-library")

            extensions.configure<LibraryExtension> { testOptions.animationsDisabled = true }

            dependencies {
                "implementation"(project(":lib:model"))
                "implementation"(project(":lib:ui:theme"))
                "implementation"(project(":lib:ui:designsystem"))
                "implementation"(project(":lib:ui:component"))
                "implementation"(project(":lib:ui:tag"))
                "implementation"(project(":lib:core"))

//                "implementation"(libs.findLibrary("androidx.lifecycle.runtimeCompose").get())
//                "implementation"(libs.findLibrary("androidx.lifecycle.viewModelCompose").get())
            }
        }
    }
}
