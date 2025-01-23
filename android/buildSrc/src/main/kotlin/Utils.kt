import java.util.*
import org.gradle.api.Project

fun Project.generateVersionCode(localProperties: Properties): Int {
    return localProperties.getProperty("OVERRIDE_VERSION_CODE")?.toIntOrNull()
        ?: execVersionCodeCargoCommand()
}

fun Project.generateVersionName(localProperties: Properties): String {
    return localProperties.getProperty("OVERRIDE_VERSION_NAME") ?: execVersionNameCargoCommand()
}

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
