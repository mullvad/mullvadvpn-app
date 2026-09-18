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

fun Project.getStringProperty(name: String): String = providers.gradleProperty(name).get()

fun Project.getIntProperty(name: String): Int = providers.gradleProperty(name).get().toInt()

fun Project.getBooleanProperty(name: String): Boolean =
    providers.gradleProperty(name).get().toBooleanStrict()

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

@UntrackedTask(because = "Always runs post build checks")
abstract class PostBuildTask @Inject constructor(private val execOperations: ExecOperations) :
    DefaultTask() {
    @get:Input abstract val skipDirtyCheck: Property<Boolean>

    @TaskAction
    fun run() {
        if (!skipDirtyCheck.get()) checkCleanWorkingDirectory(execOperations)
    }
}

fun Project.registerReleaseTask(
    releaseTaskName: String,
    appVersion: AppVersion,
    taskList: List<String>,
    expectedArtifacts: List<String>,
    skipDirtyCheck: Boolean = false,
) {
    tasks.register<PostBuildTask>("${releaseTaskName}PostBuild") {
        this.skipDirtyCheck.set(skipDirtyCheck)
        dependsOn(taskList)
    }

    val verifyArtifacts =
        registerReleaseVerifyArtifactsTask(releaseTaskName, appVersion.name, expectedArtifacts)

    tasks.register(releaseTaskName) { dependsOn(":cleanAll", verifyArtifacts) }
}

// Fetch a string and that is split by `,` into a list of strings
const val STRING_LIST_SEPARATOR = ','

fun Project.getStringListProperty(name: String): List<String> =
    providers.gradleProperty(name).get().split(STRING_LIST_SEPARATOR)
