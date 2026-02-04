import com.android.build.api.dsl.CommonExtension
import org.gradle.api.Project
import org.gradle.kotlin.dsl.dependencies

internal fun Project.configureAndroidCompose(
    commonExtension: CommonExtension<*, *, *, *, *, *>,
) {
    commonExtension.apply {
        buildFeatures.apply {
            compose = true
        }

        dependencies {
            "implementation"(libs.findLibrary("androidx.ktx"))
            "implementation"(libs.findLibrary("androidx.appcompat"))
            "implementation"(libs.findLibrary("arrow"))
            "implementation"(libs.findLibrary("material"))
            "implementation"(libs.findLibrary("compose.foundation"))
            "implementation"(libs.findLibrary("compose.material3"))
            "implementation"(libs.findLibrary("compose.ui"))
            "implementation"(libs.findLibrary("compose.ui.tooling.preview"))
            "implementation"(libs.findLibrary("compose.destinations"))
            "ksp"(libs.findLibrary("compose.destinations.ksp"))
            "testImplementation"(libs.findLibrary("junit"))
            "androidTestImplementation"(libs.findLibrary("androidx.junit"))
            "androidTestImplementation"(libs.findLibrary("androidx.espresso"))
        }
    }
}
