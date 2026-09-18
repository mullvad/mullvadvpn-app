package utilities

import org.gradle.api.Project
import org.gradle.api.provider.Provider

// This value represent a version code that would be generated from and after year 2030.
private const val MAX_ALLOWED_VERSION_CODE = 30000000

@JvmInline
value class AppVersionName(val value: String) {
    val isAlpha: Boolean
        get() = value.contains("-alpha")

    val isBeta: Boolean
        get() = value.contains("-beta")

    val isDev: Boolean
        get() = value.contains("-dev-")

    val isStable: Boolean
        get() = !isAlpha && !isBeta && !isDev

    override fun toString() = value
}

data class AppVersion(val name: AppVersionName, val code: Int) {
    val isAlpha: Boolean
        get() = name.isAlpha

    val isBeta: Boolean
        get() = name.isBeta

    val isDev: Boolean
        get() = name.isDev

    val isStable: Boolean
        get() = name.isStable

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
            name =
                AppVersionName(
                    getStringPropertyOrNull("mullvad.app.config.override.versionName")
                        ?: execVersionNameCargoCommand()
                ),
            code =
                getIntPropertyOrNull("mullvad.app.config.override.versionCode")
                    ?: execVersionCodeCargoCommand(),
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
