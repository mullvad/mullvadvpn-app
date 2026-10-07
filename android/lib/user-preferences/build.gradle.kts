plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.protobuf.core)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.userpreferences"

    buildFeatures { buildConfig = true }

    kotlin {
        compilerOptions {
            // Protobuf Kotlin generator emits explicit visibility modifiers,
            // so we exclude that check.
            freeCompilerArgs.add("-Xwarning-level=REDUNDANT_VISIBILITY_MODIFIER:disabled")
        }
    }
}

protobuf {
    protoc { artifact = libs.plugins.protobuf.protoc.get().toString() }
    generateProtoTasks {
        all().forEach {
            it.builtins {
                create("java") { option("lite") }
                create("kotlin") { option("lite") }
            }
        }
    }
}

dependencies {
    implementation(projects.lib.model)

    implementation(libs.androidx.datastore)
    implementation(libs.protobuf.kotlin.lite)
}
