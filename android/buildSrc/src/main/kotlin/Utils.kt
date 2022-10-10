import java.io.ByteArrayOutputStream
import org.gradle.api.Project
import org.gradle.process.ExecSpec

fun Project.execWithOutput(spec: ExecSpec.() -> Unit) =
    ByteArrayOutputStream().use { outputStream ->
        exec {
            this.spec()
            this.standardOutput = outputStream
        }
        outputStream.toString().trim()
    }

fun Project.generateVersionCode() = execWithOutput {
    commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionCode")
}.toInt()

fun Project.generateVersionName() = execWithOutput {
    commandLine("cargo", "run", "-q", "--bin", "mullvad-version", "versionName")
}
