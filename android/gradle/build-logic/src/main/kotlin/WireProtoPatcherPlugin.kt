import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction

/** Configuration for [WireProtoPatcherPlugin]. */
abstract class WireProtoPatcherExtension {
    /** Directory containing the original, unpatched `.proto` files. */
    abstract val protoSourceDir: DirectoryProperty
    /** File containing the wrapped types, e.g., `WireBoolValue`, `WireInt32Value`, etc. */
    abstract val wireWrappersProtoFile: Property<File>
}

class WireProtoPatcherPlugin : Plugin<Project> {
    override fun apply(project: Project) {
        val extension =
            project.extensions.create("wireProtoPatcher", WireProtoPatcherExtension::class.java)

        // Register task
        val patchTask =
            project.tasks.register("patchProtoFiles", PatchProtosTask::class.java) {
                sourceDir.set(extension.protoSourceDir)
                wrappersFile.set(extension.wireWrappersProtoFile)
                outputDir.set(project.layout.buildDirectory.dir("generated/patched_protos"))
            }

        // Wait for the Wire plugin to be applied in the consumer project
        project.pluginManager.withPlugin("com.squareup.wire") {

            // Auto-configure Wire's Task dependency using class name reflection
            // (so we don't have to add Wire as a classpath dependency in buildLogic)
            project.tasks.configureEach {
                if (this.javaClass.name.startsWith("com.squareup.wire.gradle.WireTask")) {
                    dependsOn(patchTask)
                }
            }
        }

        project.afterEvaluate {
            check(extension.protoSourceDir.isPresent) {
                "wireProtoPatcher.protoSourceDir must be set"
            }
            check(extension.wireWrappersProtoFile.isPresent) {
                "wireProtoPatcher.wireWrappersProtoFile must be set"
            }
        }
    }
}

abstract class PatchProtosTask : DefaultTask() {

    @get:InputDirectory
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sourceDir: DirectoryProperty

    @get:Input abstract val wrappersFile: Property<File>

    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @TaskAction
    fun patchProtos() {
        val outDirFile = outputDir.get().asFile
        outDirFile.mkdirs()

        // Copy the wrappers file to the output directory
        val safeWrappersFile = File(outDirFile, wrappersFile.get().name)
        wrappersFile.get().copyTo(safeWrappersFile, overwrite = true)

        // Process all original proto files
        val srcDirFile = sourceDir.get().asFile
        srcDirFile
            .walkTopDown()
            .filter { it.isFile && it.extension == "proto" }
            .forEach { protoFile ->
                var content = protoFile.readText()

                // Replace the import statements
                content =
                    content.replace(
                        Regex("""import\s+"google/protobuf/wrappers\.proto"\s*;"""),
                        """import "wire_wrappers.proto";""",
                    )

                // Replace the usage of types (e.g., google.protobuf.BoolValue ->
                // safe_wrappers.WireBoolValue)
                content =
                    content.replace(Regex("""google\.protobuf\.([A-Za-z0-9]+Value)""")) {
                        matchResult ->
                        "wire_wrappers.Wire${matchResult.groupValues[1]}"
                    }

                // Save modified file into output directory
                val relativePath = protoFile.relativeTo(srcDirFile).path
                val outputFile = File(outDirFile, relativePath)
                outputFile.parentFile.mkdirs()
                outputFile.writeText(content)
            }
    }
}
