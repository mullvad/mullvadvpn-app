package utilities

import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.zip.ZipFile
import org.gradle.api.DefaultTask
import org.gradle.api.Project
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.TaskProvider
import org.gradle.api.tasks.UntrackedTask
import org.gradle.kotlin.dsl.register

private const val RELAY_LIST_ASSET = "assets/relays.json"
private const val BUNDLE_RELAY_LIST_ASSET = "base/assets/relays.json"

@UntrackedTask(because = "Always verifies the produced artifacts")
abstract class VerifyArtifactsTask : DefaultTask() {
    @get:Internal abstract val artifacts: ConfigurableFileCollection

    @get:Input abstract val versionName: Property<AppVersionName>

    @get:Input abstract val distDirectory: Property<String>

    @TaskAction
    fun run() {
        val artifactFiles = artifacts.files.sortedBy { it.name }
        val releaseVersion = versionName.get()
        val missingArtifacts = artifactFiles.filterNot { it.isFile }

        check(missingArtifacts.isEmpty()) {
            "Missing artifacts:\n" + missingArtifacts.joinToString("\n") { "  ${it.path}" }
        }

        println("\nVerifying artifacts for version $releaseVersion")
        println("  Directory: ${distDirectory.get()}")
        println("\nExpected artifacts:")
        artifactFiles.forEach { println("  ${it.name}") }

        artifactFiles.forEach { verifyArtifact(it, requireRelayList = !releaseVersion.isDev) }
    }

    private fun verifyArtifact(artifact: File, requireRelayList: Boolean) {
        val relayListSize = relayListSize(artifact)

        if (requireRelayList) {
            checkNotNull(relayListSize) { "${artifact.name} does not bundle a relay list" }
            check(relayListSize > 0) { "${artifact.name} bundles an empty relay list" }
        }

        val relayList =
            when {
                relayListSize == null -> "missing"
                relayListSize == 0L -> "empty"
                else -> "present ($relayListSize bytes)"
            }

        println("\nVerifying ${artifact.name}")
        println("  Checksum:   ${artifact.sha256()} (sha256)")
        println("  Relay list: $relayList")
    }

    private fun relayListSize(artifact: File): Long? =
        openArchive(artifact).use { zip ->
            val relayListAsset =
                if (artifact.extension == "aab") BUNDLE_RELAY_LIST_ASSET else RELAY_LIST_ASSET

            zip.getEntry(relayListAsset)?.size
        }

    private fun openArchive(artifact: File): ZipFile =
        try {
            ZipFile(artifact)
        } catch (exception: IOException) {
            error("${artifact.name} is not a readable archive: ${exception.message}")
        }
}

private fun File.sha256(): String {
    val digest = MessageDigest.getInstance("SHA-256")
    inputStream().use { stream ->
        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
        var bytes = stream.read(buffer)
        while (bytes != -1) {
            digest.update(buffer, 0, bytes)
            bytes = stream.read(buffer)
        }
    }
    return digest.digest().joinToString("") { "%02x".format(it) }
}

private fun Project.distDirectory(): File = rootDir.parentFile.resolve("dist")

private fun Project.distArtifacts(artifactNames: List<String>): List<File> {
    val distDirectory = distDirectory()

    return artifactNames.map { distDirectory.resolve(it) }
}

internal fun Project.registerReleaseVerifyArtifactsTask(
    releaseTaskName: String,
    versionName: AppVersionName,
    expectedArtifacts: List<String>,
): TaskProvider<VerifyArtifactsTask> =
    tasks.register<VerifyArtifactsTask>("${releaseTaskName}VerifyArtifacts") {
        this.versionName.set(versionName)
        this.distDirectory.set(distDirectory().path)
        artifacts.from(distArtifacts(expectedArtifacts))
        dependsOn("${releaseTaskName}PostBuild")
    }
