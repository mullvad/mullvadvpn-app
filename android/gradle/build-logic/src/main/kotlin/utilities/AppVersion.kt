package utilities

import org.gradle.api.Project
import org.gradle.api.provider.Provider

// This value represent a version code that would be generated from and after year 2030.
private const val MAX_ALLOWED_VERSION_CODE = 30000000

data class AppVersion(val code: Int, val name: String) {
    val isAlpha: Boolean
        get() = name.contains("-alpha")

    val isBeta: Boolean
        get() = name.contains("-beta")

    val isDev: Boolean
        get() = name.contains("-dev-")

    val isStable: Boolean
        get() = !isAlpha && !isBeta && !isDev

    init {
        // This is a safety net to avoid generating too big version codes, since that could
        // potentially be hard and inconvenient to recover from.
        require(code <= MAX_ALLOWED_VERSION_CODE) {
            "versionCode ($code) must be <= $MAX_ALLOWED_VERSION_CODE"
        }
    }
}

val Project.appVersionProvider: Provider<AppVersion>
    get() = provider {
        AppVersion(
            code =
                getIntPropertyOrNull("mullvad.app.config.override.versionCode")
                    ?: execVersionCodeCargoCommand(),
            name =
                getStringPropertyOrNull("mullvad.app.config.override.versionName")
                    ?: execVersionNameCargoCommand(),
        )
    }

private fun Project.execVersionCodeCargoCommand() =
    providers
        .exec { commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionCode") }
        .standardOutput
        .asText
        .get()
        .trim()
        .toInt()

private fun Project.execVersionNameCargoCommand() =
    providers
        .exec { commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionName") }
        .standardOutput
        .asText
        .get()
        .trim()
