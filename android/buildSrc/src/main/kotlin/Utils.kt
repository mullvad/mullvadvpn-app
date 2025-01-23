import java.io.ByteArrayOutputStream
import java.util.*
import org.gradle.api.Project
import org.gradle.process.ExecSpec

fun Project.generateVersionCode(localProperties: Properties): Int {
    return localProperties.getProperty("OVERRIDE_VERSION_CODE")?.toIntOrNull()
        ?: execVersionCodeCargoCommand()
}

fun Project.generateVersionName(localProperties: Properties): String {
    return localProperties.getProperty("OVERRIDE_VERSION_NAME") ?: execVersionNameCargoCommand()
}

fun Project.generateRemapArguments(): String {
    return providers.exec {
        commandLine("cargo", "run", "-p", "remap-path-prefix", "-q")
    }.standardOutput.asText.get().trim()
}

private fun Project.execVersionCodeCargoCommand() =
    providers.exec {
        commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionCode")
    }.standardOutput.asText.get().trim().toInt()

private fun Project.execVersionNameCargoCommand() =
    providers.exec {
        commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionName")
    }.standardOutput.asText.get().trim()
