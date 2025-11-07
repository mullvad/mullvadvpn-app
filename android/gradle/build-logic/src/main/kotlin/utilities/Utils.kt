package utilities

import org.gradle.api.Project

// This is a hack and will not work correctly under all scenarios.
// See DROID-1696 for how we can improve this.
fun Project.isReleaseBuild() =
    gradle.startParameter.getTaskNames().any {
        it.contains("release", ignoreCase = true) || it.contains("fdroid", ignoreCase = true)
    }

fun Project.isAlphaBuild(): Boolean {
    val versionName = generateVersionName()
    return versionName.contains("alpha")
}

fun Project.isDevBuild(): Boolean {
    val versionName = generateVersionName()
    return versionName.contains("-dev-")
}

fun Project.generateVersionCode(): Int {
    val versionCode =
        getIntPropertyOrNull("mullvad.app.config.override.versionCode")
            ?: execVersionCodeCargoCommand()
    // This is a safety net to avoid generating too big version codes, since that could potentially
    // be hard and inconvenient to recover from.
    require(versionCode <= MAX_ALLOWED_VERSION_CODE) {
        "versionCode ($versionCode) must be <= $MAX_ALLOWED_VERSION_CODE"
    }
    return versionCode
}

fun Project.generateVersionName(): String =
    getStringPropertyOrNull("mullvad.app.config.override.versionName")
        ?: execVersionNameCargoCommand()

fun Project.generateRemapArguments(): String {
    val script = "${projectDir.parent}/../building/rustc-remap-path-prefix.sh"
    return providers.exec { commandLine(script) }.standardOutput.asText.get().trim()
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

fun Project.getStringPropertyOrNull(name: String): String? = findProperty(name)?.toString()

fun Project.getIntPropertyOrNull(name: String): Int? = findProperty(name)?.toString()?.toInt()

fun Project.getBooleanPropertyOrNull(name: String): Boolean? =
    findProperty(name)?.toString()?.toBooleanStrict()

fun Project.getStringProperty(name: String): String = properties[name].toString()

fun Project.getIntProperty(name: String): Int = properties[name].toString().toInt()

fun Project.getBooleanProperty(name: String): Boolean =
    properties[name].toString().toBooleanStrict()

// Fetch a string and that is split by `,` into a list of strings
const val STRING_LIST_SEPARATOR = ','

fun Project.getStringListProperty(name: String): List<String> =
    properties[name].toString().split(STRING_LIST_SEPARATOR)
