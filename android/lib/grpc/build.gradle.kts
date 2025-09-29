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

wireProtoPatcher {
    protoSourceDir.set(file("${rootProject.projectDir}/../mullvad-management-interface/proto"))
}

wire {
    sourcePath {
        val patchedDir =
            tasks.named("patchProtoFiles").flatMap { (it as PatchProtosTask).outputDir }
        srcDir(patchedDir)
    }

    kotlin {
        android = true
        rpcRole = "client"
        rpcCallStyle = "suspending"
        explicitStreamingCalls = true
        emitProtoReader32 = true
        escapeKotlinKeywords = true
        makeImmutableCopies = true
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
