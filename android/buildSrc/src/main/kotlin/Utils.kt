import java.util.*
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

fun Project.generateVersionCode(): Int =
    getMullvadProperty("app.config.override.versionCode")?.also { println("GOT PROPERTY :$it") }?.toInt()
        ?: execVersionCodeCargoCommand()

fun Project.generateVersionName(): String =
    getMullvadProperty("app.config.override.versionName")
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

private lateinit var mullvadProperties: Properties

fun Project.getMullvadProperty(name: String): String? {
    if (!::mullvadProperties.isInitialized) {
        mullvadProperties = loadMullvadProperties()
    }

    return System.getenv(name)
        ?: rootProject.properties.getOrDefault(name, null) as? String
        ?: mullvadProperties.getProperty(name, null)
}

private fun Project.loadMullvadProperties(): Properties {
    val props = Properties()
    props.load(rootProject.file("mullvad.properties").inputStream())
    props.toMutableMap()
    return props
}
