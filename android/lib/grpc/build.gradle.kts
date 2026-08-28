plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.mullvad.unit.test)
    alias(libs.plugins.wire)
    alias(libs.plugins.wire.proto.patcher)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.grpc"

    kotlin {
        compilerOptions {
            freeCompilerArgs.add("-XXLanguage:+WhenGuards")
            // This is due to a warning in the generated code from Wire.
            allWarningsAsErrors = false
        }
    }
}

wire {
    sourcePath {
        // Gradle is smart enough to resolve the task's outputDir property
        val patchedDir = tasks.named("patchProtoFiles").flatMap {
            (it as PatchProtosTask).outputDir
        }
        srcDir(patchedDir)
    }

    kotlin {
        android = true
        rpcRole = "client"
        rpcCallStyle = "blocking"
        explicitStreamingCalls = true
        emitProtoReader32 = true
    }
}

dependencies {
    implementation(projects.lib.common)
    implementation(projects.lib.model)
    implementation(projects.lib.talpid)

    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines)
    implementation(libs.kotlinx.coroutines.android)

    implementation(libs.wire.runtime)
    implementation(libs.wire.grpc)
    implementation(libs.okhttp.logging.interceptor)

    implementation(libs.arrow)
    implementation(libs.arrow.optics)
}
