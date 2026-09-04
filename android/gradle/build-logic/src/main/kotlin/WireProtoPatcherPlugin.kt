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

private const val SAFE_WRAPPERS_PROTO_FILE = "safe_wrappers.proto"

/** Configuration for [WireProtoPatcherPlugin]. */
abstract class WireProtoPatcherExtension {
    /** Directory containing the original, unpatched `.proto` files. */
    abstract val protoSourceDir: DirectoryProperty
}

class WireProtoPatcherPlugin : Plugin<Project> {
    override fun apply(project: Project) {
        val extension =
            project.extensions.create("wireProtoPatcher", WireProtoPatcherExtension::class.java)

        // Read the bundled safe_wrappers.proto once at configuration time so that its content is
        // tracked as a task input, this way the task is correctly re-run if the file's content is
        // changed.
        val safeWrappersProtoContent =
            WireProtoPatcherPlugin::class
                .java
                .classLoader
                .getResourceAsStream(SAFE_WRAPPERS_PROTO_FILE)
                ?.bufferedReader()
                ?.readText() ?: error("Could not find $SAFE_WRAPPERS_PROTO_FILE")

        // Register task
        val patchTask =
            project.tasks.register("patchProtoFiles", PatchProtosTask::class.java) {
                sourceDir.set(extension.protoSourceDir)
                safeWrappersContent.set(safeWrappersProtoContent)
                outputDir.set(project.layout.buildDirectory.dir("generated/patched_protos"))
            }

        // 2. Wait for the Wire plugin to be applied in the consumer project
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
        }
    }
}

abstract class PatchProtosTask : DefaultTask() {

    @get:InputDirectory
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sourceDir: DirectoryProperty

    /** Content of the bundled `safe_wrappers.proto` resource, see [WireProtoPatcherPlugin]. */
    @get:Input abstract val safeWrappersContent: Property<String>

    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @TaskAction
    fun patchProtos() {
        val outDirFile = outputDir.get().asFile
        outDirFile.mkdirs()

        // 1. Write out the statically defined safe_wrappers.proto file
        val safeWrappersFile = File(outDirFile, SAFE_WRAPPERS_PROTO_FILE)
        safeWrappersFile.writeText(safeWrappersContent.get())

        // 2. Process all original proto files
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
                        """import "safe_wrappers.proto";""",
                    )

                // Replace the usage of types (e.g., google.protobuf.BoolValue ->
                // safe_wrappers.SafeBoolValue)
                content =
                    content.replace(Regex("""google\.protobuf\.([A-Za-z0-9]+Value)""")) {
                        matchResult ->
                        "safe_wrappers.Safe${matchResult.groupValues[1]}"
                    }

                // 3. Save modified file into output directory
                val relativePath = protoFile.relativeTo(srcDirFile).path
                val outputFile = File(outDirFile, relativePath)
                outputFile.parentFile.mkdirs()
                outputFile.writeText(content)
            }
    }
}
