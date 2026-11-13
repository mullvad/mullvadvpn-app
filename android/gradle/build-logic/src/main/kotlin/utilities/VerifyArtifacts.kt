package utilities

import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.zip.ZipFile
import org.gradle.api.DefaultTask
import org.gradle.api.Project
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Copy
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.TaskProvider
import org.gradle.api.tasks.UntrackedTask
import org.gradle.kotlin.dsl.register
import org.gradle.kotlin.dsl.withType

private const val MIB = 1024L * 1024

private sealed class Artifact(private val file: File, private val relayListAsset: String) {
    class Apk(file: File) : Artifact(file, "assets/relays.json")

    class Bundle(file: File) : Artifact(file, "base/assets/relays.json")

    val name: String = file.name
    val size: Long = file.length()

    fun sha256(): String = file.sha256()

    fun relayListSize(): Long? = openArchive().use { zip -> zip.getEntry(relayListAsset)?.size }

    private fun openArchive(): ZipFile =
        try {
            ZipFile(file)
        } catch (exception: IOException) {
            error("$name is not a readable archive: ${exception.message}")
        }

    companion object {
        fun of(file: File): Artifact =
            when (file.extension) {
                "apk" -> Apk(file)
                "aab" -> Bundle(file)
                else -> error("${file.name} is not a supported artifact type")
            }
    }
}

private data class ArtifactRequirements(val maxSize: Long, val requiresRelayList: Boolean) {
    companion object {
        fun of(artifact: Artifact, version: AppVersionName?) =
            ArtifactRequirements(
                maxSize =
                    when (artifact) {
                        is Artifact.Apk -> 42 * MIB
                        is Artifact.Bundle -> 70 * MIB
                    },
                requiresRelayList = version?.isDev != true,
            )
    }
}

@UntrackedTask(because = "Always verifies the produced artifacts")
abstract class VerifyArtifactsTask : DefaultTask() {
    @get:Internal abstract val artifactFiles: ConfigurableFileCollection

    @get:Optional @get:Input abstract val versionName: Property<AppVersionName>

    @get:Optional @get:Input abstract val distDirectory: Property<String>

    @TaskAction
    fun run() {
        val releaseVersion = versionName.orNull
        val missingFiles = artifactFiles.filterNot { it.isFile }

        check(missingFiles.isEmpty()) {
            "Missing artifacts:\n" + missingFiles.joinToString("\n") { "  ${it.path}" }
        }

        val artifacts = artifactFiles.map(Artifact::of).sortedBy { it.name }

        if (releaseVersion != null) {
            println("\nVerifying artifacts for version $releaseVersion")
            println("  Directory: ${distDirectory.get()}")
            println("\nExpected artifacts:")
            artifacts.forEach { println("  ${it.name}") }
        }

        artifacts.forEach { artifact ->
            verifyArtifact(artifact, ArtifactRequirements.of(artifact, releaseVersion))
        }
    }

    private fun verifyArtifact(artifact: Artifact, requirements: ArtifactRequirements) {
        check(artifact.size <= requirements.maxSize) {
            "${artifact.name} is ${artifact.size} bytes, the limit is ${requirements.maxSize} bytes"
        }

        val relayListSize = artifact.relayListSize()

        if (requirements.requiresRelayList) {
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
        println("  Size:       ${artifact.size} bytes")
        println("  Relay list: $relayList")
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
        artifactFiles.from(distArtifacts(expectedArtifacts))
        dependsOn("${releaseTaskName}PostBuild")
    }

fun Project.registerVerifyArtifactsTask(versionName: AppVersionName) {
    val artifactPath = providers.gradleProperty("artifact").orNull
    val releaseVersion =
        providers.gradleProperty("release").orNull?.let { AppVersionName(it) } ?: versionName

    tasks.register<VerifyArtifactsTask>("verifyArtifacts") {
        if (artifactPath != null) {
            artifactFiles.from(File(artifactPath))
        } else {
            this.versionName.set(releaseVersion)
            this.distDirectory.set(distDirectory().path)
            artifactFiles.from(distArtifacts(fullReleaseArtifacts(releaseVersion)))
            mustRunAfter(tasks.withType<Copy>())
        }
    }
}
