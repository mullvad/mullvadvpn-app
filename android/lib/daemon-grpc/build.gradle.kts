import com.google.protobuf.gradle.proto

plugins {
    id("mullvad.android-library")
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.protobuf.core)
    alias(libs.plugins.junit5.android)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.daemon.grpc"

    sourceSets {
        getByName("main") {
            proto { srcDir("${rootProject.projectDir}/../mullvad-management-interface/proto") }
        }
    }

    kotlin { compilerOptions { freeCompilerArgs.add("-XXLanguage:+WhenGuards") } }
}

protobuf {
    protoc { artifact = libs.plugins.protobuf.protoc.get().toString() }
    plugins {
        val grpcPluginPath = System.getenv("PROTOC_GEN_GRPC_JAVA_PLUGIN")
        create("java") {
            if (grpcPluginPath != null) {
                path = grpcPluginPath
            } else {
                artifact = libs.plugins.grpc.protoc.gen.grpc.java.get().toString()
            }
        }
        create("grpc") {
            if (grpcPluginPath != null) {
                path = grpcPluginPath
            } else {
                artifact = libs.plugins.grpc.protoc.gen.grpc.java.get().toString()
            }
        }
        create("grpckt") {
            artifact = libs.plugins.grpc.protoc.gen.grpc.kotlin.get().toString() + ":jdk8@jar"
        }
    }
    generateProtoTasks {
        all().forEach {
            it.plugins {
                create("java") { option("lite") }
                create("grpc") { option("lite") }
                create("grpckt") { option("lite") }
            }
            it.builtins { create("kotlin") { option("lite") } }
        }
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

    implementation(libs.grpc.okhttp)
    implementation(libs.grpc.android)
    implementation(libs.grpc.stub)
    implementation(libs.grpc.kotlin.stub)
    implementation(libs.grpc.protobuf.lite)
    implementation(libs.protobuf.kotlin.lite)

    implementation(libs.arrow)
    implementation(libs.arrow.optics)

    testImplementation(projects.lib.commonTest)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.turbine)
    testImplementation(libs.junit.jupiter.api)
    testRuntimeOnly(libs.junit.jupiter.engine)
    testImplementation(libs.junit.jupiter.params)
}
