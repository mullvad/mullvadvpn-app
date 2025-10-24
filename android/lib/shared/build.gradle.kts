import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.junit5.android)
    alias(libs.plugins.protobuf.core)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.shared"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()

    defaultConfig { minSdk = libs.versions.min.sdk.get().toInt() }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlin {
        compilerOptions {
            jvmTarget = JvmTarget.fromTarget(libs.versions.jvm.target.get())
            allWarningsAsErrors = true
        }
    }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
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
    implementation(projects.lib.resource)
    implementation(projects.lib.common)
    implementation(projects.lib.daemonGrpc)
    implementation(projects.lib.model)

    implementation(libs.arrow)
    implementation(libs.kermit)
    implementation(libs.kotlin.stdlib)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.androidx.datastore)
    implementation(libs.protobuf.kotlin.lite)

    testImplementation(libs.kotlin.test)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk)
    testImplementation(libs.junit.jupiter.api)
    testImplementation(libs.junit.jupiter.params)
    testImplementation(libs.turbine)
    testImplementation(projects.lib.commonTest)
    testRuntimeOnly(libs.junit.jupiter.engine)
}
