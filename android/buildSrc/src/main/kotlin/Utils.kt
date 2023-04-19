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

private fun Project.execVersionCodeCargoCommand() =
    execWithOutput { commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionCode") }
        .toInt()

private fun Project.execVersionNameCargoCommand() = execWithOutput {
    commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionName")
}

private fun Project.execWithOutput(spec: ExecSpec.() -> Unit) =
    ByteArrayOutputStream().use { outputStream ->
        exec {
            this.spec()
            this.standardOutput = outputStream
        }
        outputStream.toString().trim()
    }
