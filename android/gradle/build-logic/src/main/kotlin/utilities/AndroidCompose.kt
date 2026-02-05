package utilities

import com.android.build.api.dsl.CommonExtension
import org.gradle.api.Project
import org.gradle.kotlin.dsl.dependencies

internal fun Project.configureAndroidCompose(commonExtension: CommonExtension<*, *, *, *, *, *>) {
    commonExtension.apply {
        buildFeatures.apply { compose = true }

        dependencies {
            "implementation"(libs.findLibrary("compose-foundation").get())
            "implementation"(libs.findLibrary("compose-material3").get())
            "implementation"(libs.findLibrary("compose-ui").get())
            "implementation"(libs.findLibrary("compose-ui-tooling-preview").get())
            "debugImplementation"(libs.findLibrary("compose-ui-tooling").get())

            "androidTestImplementation"(libs.findLibrary("junit5-android-test-compose").get())
            "androidTestImplementation"(libs.findLibrary("mockk").get())
            "androidTestImplementation"(libs.findLibrary("mockk-android").get())
            "androidTestImplementation"(libs.findLibrary("androidx-espresso").get())
            "androidTestImplementation"(libs.findLibrary("junit-jupiter-api").get())
            "androidTestRuntimeOnly"(libs.findLibrary("junit-jupiter-engine").get())

            "androidTestImplementation"(project(":lib:screen-test"))
        }
    }
}
