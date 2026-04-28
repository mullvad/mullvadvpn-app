package utilities

import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.Project
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.UntrackedTask
import org.gradle.kotlin.dsl.register
import org.gradle.process.ExecOperations

private const val BUILD_HEADER_LINE_LENGTH = 60

// This is a hack and will not work correctly under all scenarios.
// See DROID-1696 for how we can improve this.
fun Project.isReleaseBuild() =
    gradle.startParameter.getTaskNames().any { it.contains("release", ignoreCase = true) }

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

fun checkCleanWorkingDirectory(execOperations: ExecOperations) {
    val output = java.io.ByteArrayOutputStream()
    execOperations.exec {
        commandLine("git", "status", "--porcelain")
        standardOutput = output
    }
    if (output.toString().trim().isNotBlank()) error("Dirty working directory!")
}

fun printBuildHeader(versionName: String) {
    val hostname = java.net.InetAddress.getLocalHost().hostName
    val line = "=".repeat(BUILD_HEADER_LINE_LENGTH)
    println("$line\nBuilding Mullvad VPN $versionName on $hostname\n$line")
}

fun printBuildChecksums(versionName: String, distDir: java.io.File) {
    val artifacts =
        distDir
            .listFiles { f -> f.name.startsWith("MullvadVPN-$versionName") }
            ?.sortedBy { it.name }
    check(!artifacts.isNullOrEmpty()) {
        "No artifacts found in $distDir for MullvadVPN-$versionName"
    }
    println("\nBuild checksums:")
    artifacts.forEach { artifact ->
        val digest = java.security.MessageDigest.getInstance("SHA-256")
        artifact.inputStream().use { stream ->
            val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
            var bytes = stream.read(buffer)
            while (bytes != -1) {
                digest.update(buffer, 0, bytes)
                bytes = stream.read(buffer)
            }
        }
        println("  ${digest.digest().joinToString("") { "%02x".format(it) }}  ${artifact.name}")
    }
}

@UntrackedTask(because = "Always runs preflight checks")
abstract class PreBuildTask @Inject constructor(private val execOperations: ExecOperations) :
    DefaultTask() {
    @get:Input abstract val skipDirtyCheck: Property<Boolean>
    @get:Input abstract val versionName: Property<String>

    @TaskAction
    fun run() {
        if (!skipDirtyCheck.get()) checkCleanWorkingDirectory(execOperations)
        printBuildHeader(versionName.get())
    }
}

@UntrackedTask(because = "Always prints build checksums")
abstract class PostBuildTask @Inject constructor(private val execOperations: ExecOperations) :
    DefaultTask() {
    @get:Input abstract val skipDirtyCheck: Property<Boolean>
    @get:Input abstract val versionName: Property<String>
    @get:Input abstract val distDirPath: Property<String>

    @TaskAction
    fun run() {
        if (!skipDirtyCheck.get()) checkCleanWorkingDirectory(execOperations)
        printBuildChecksums(versionName.get(), java.io.File(distDirPath.get()))
    }
}

fun Project.registerReleaseTask(
    releaseName: String,
    appVersion: AppVersion,
    taskList: List<String>,
    skipClean: Boolean = false,
    skipDirtyCheck: Boolean = false,
) {
    val releaseVersionName = appVersion.name
    val releaseDistDirPath = rootDir.parentFile.resolve("dist").absolutePath

    if (!skipClean) tasks.configureEach { if (releaseName in taskList) dependsOn("clean") }

    val postBuild =
        tasks.register<PostBuildTask>("${releaseName}PostBuild") {
            this.skipDirtyCheck.set(skipDirtyCheck)
            this.versionName.set(releaseVersionName)
            this.distDirPath.set(releaseDistDirPath)
            dependsOn(taskList)
        }

    tasks.register(releaseName) { dependsOn(postBuild) }
}

// Fetch a string and that is split by `,` into a list of strings
const val STRING_LIST_SEPARATOR = ','

fun Project.getStringListProperty(name: String): List<String> =
    properties[name].toString().split(STRING_LIST_SEPARATOR)
