package utilities

import org.gradle.api.Project

// This is a hack and will not work correctly under all scenarios.
// See DROID-1696 for how we can improve this.
fun Project.isReleaseBuild() =
    gradle.startParameter.getTaskNames().any {
        it.contains("release", ignoreCase = true) || it.contains("fdroid", ignoreCase = true)
    }

fun Project.generateRemapArguments(): String {
    val script = "${projectDir.parent}/../building/rustc-remap-path-prefix.sh"
    return providers.exec { commandLine(script) }.standardOutput.asText.get().trim()
}

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
