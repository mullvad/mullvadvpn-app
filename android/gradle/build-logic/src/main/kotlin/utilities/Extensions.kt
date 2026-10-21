package utilities

import com.android.build.api.dsl.Lint
import org.gradle.api.Project
import org.gradle.api.artifacts.Dependency
import org.gradle.api.artifacts.dsl.DependencyHandler

fun String.isNonStableVersion(): Boolean {
    val nonStableQualifiers = listOf("alpha", "beta", "rc")

    val isNonStable =
        nonStableQualifiers
            .map { qualifier -> Regex("(?i).*[.-]$qualifier[.\\d-+]*") }
            .any { it.matches(this) }

    return isNonStable
}

fun DependencyHandler.leakCanaryImplementation(dependencyNotation: Any): Dependency? =
    add("leakCanaryImplementation", dependencyNotation)

fun DependencyHandler.playImplementation(dependencyNotation: Any): Dependency? =
    add("playImplementation", dependencyNotation)

fun Lint.configureMissingTranslationCheck(project: Project, appVersion: AppVersion) {
    val failOnMissing = project.getBooleanProperty("mullvad.app.lint.failOnMissingTranslations")
    if (appVersion.isStable || failOnMissing) {
        fatal += "MissingTranslation"
    } else {
        disable += "MissingTranslation"
    }
}
