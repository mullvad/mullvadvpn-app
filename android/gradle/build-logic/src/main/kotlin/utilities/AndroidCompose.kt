package utilities

import com.android.build.api.dsl.CommonExtension
import org.gradle.api.Project
import org.gradle.kotlin.dsl.dependencies

internal fun Project.configureAndroidCompose(commonExtension: CommonExtension<*, *, *, *, *, *>) {
    commonExtension.apply {
        buildFeatures.apply { compose = true }

        dependencies {
            "implementation"(libs.findLibrary("material").get())
            "implementation"(libs.findLibrary("compose-foundation").get())
            "implementation"(libs.findLibrary("compose-material3").get())
            "implementation"(libs.findLibrary("compose-ui").get())
            "implementation"(libs.findLibrary("compose-ui-tooling-preview").get())
            "debugImplementation"(libs.findLibrary("compose-ui-tooling").get())
            "testImplementation"(libs.findLibrary("junit").get())
            "androidTestImplementation"(libs.findLibrary("androidx-junit").get())
        }
    }
}
