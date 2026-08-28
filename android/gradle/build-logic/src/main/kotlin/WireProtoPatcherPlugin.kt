import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.apply

import org.gradle.api.DefaultTask
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import java.io.File

class WireProtoPatcherPlugin : Plugin<Project> {
    override fun apply(project: Project) {

        // 1. Register the patching task
        val patchTask = project.tasks.register("patchProtoFiles", PatchProtosTask::class.java) {
            // Set input to the non-standard proto directory
            sourceDir.set(project.layout.projectDirectory.dir("../../../mullvad-management-interface/proto"))
            // Set output to the ephemeral build directory
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
    }
}

abstract class PatchProtosTask : DefaultTask() {

    @get:InputDirectory
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sourceDir: DirectoryProperty

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @TaskAction
    fun patchProtos() {
        val outDirFile = outputDir.get().asFile
        outDirFile.mkdirs()

        // 1. Generate the safe_wrappers.proto file dynamically
        val safeWrappersFile = File(outDirFile, "safe_wrappers.proto")
        safeWrappersFile.writeText("""
            syntax = "proto3";
            package safe_wrappers;

            message SafeDoubleValue { double value = 1; }
            message SafeFloatValue { float value = 1; }
            message SafeInt64Value { int64 value = 1; }
            message SafeUInt64Value { uint64 value = 1; }
            message SafeInt32Value { int32 value = 1; }
            message SafeUInt32Value { uint32 value = 1; }
            message SafeBoolValue { bool value = 1; }
            message SafeStringValue { string value = 1; }
            message SafeBytesValue { bytes value = 1; }
        """.trimIndent())

        // 2. Process all original proto files
        val srcDirFile = sourceDir.get().asFile
        srcDirFile.walkTopDown()
            .filter { it.isFile && it.extension == "proto" }
            .forEach { protoFile ->
                var content = protoFile.readText()

                // Replace the import statement
                content = content.replace(
                    Regex("""import\s+"google/protobuf/wrappers\.proto"\s*;"""),
                    """import "safe_wrappers.proto";"""
                )

                // Replace the usage of types (e.g., google.protobuf.BoolValue -> safe_wrappers.SafeBoolValue)
                content = content.replace(
                    Regex("""google\.protobuf\.([A-Za-z0-9]+Value)""")
                ) { matchResult ->
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
