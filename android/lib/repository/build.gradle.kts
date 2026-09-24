plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.protobuf.core)
    alias(libs.plugins.mullvad.unit.test)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.repository"

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
    implementation(projects.lib.ui.resource)
    implementation(projects.lib.common)
    implementation(projects.lib.grpc)
    implementation(projects.lib.model)
    implementation(projects.lib.payment)
    implementation(projects.lib.endpoint)

    implementation(libs.arrow)
    implementation(libs.arrow.optics)
    implementation(libs.arrow.resilience)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.androidx.datastore)
    implementation(libs.androidx.lifecycle.runtime)
    implementation(libs.protobuf.kotlin.lite)
}
