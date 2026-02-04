plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.protobuf.core)
    alias(libs.plugins.mullvad.unit.test)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.repository"

    buildFeatures { buildConfig = true }
}

protobuf {
    protoc { artifact = libs.plugins.protobuf.protoc.get().toString() }
    plugins {
        create("java") { artifact = libs.plugins.grpc.protoc.gen.grpc.java.get().toString() }
    }
    generateProtoTasks {
        all().forEach {
            it.plugins { create("java") { option("lite") } }
            it.builtins { create("kotlin") { option("lite") } }
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
    implementation(libs.protobuf.kotlin.lite)
}
