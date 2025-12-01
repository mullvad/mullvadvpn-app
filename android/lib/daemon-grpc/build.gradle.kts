plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
    alias(libs.plugins.junit5.android)
    alias(libs.plugins.wire)
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.daemon.grpc"

    /*sourceSets {
        getByName("main") {
            proto { srcDir("${rootProject.projectDir}/../mullvad-management-interface/proto") }
        }
    }*/

    kotlin { compilerOptions { freeCompilerArgs.add("-XXLanguage:+WhenGuards") } }
}

wire {
    sourcePath { srcDir("${rootProject.projectDir}/../mullvad-management-interface/proto") }

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
    implementation(libs.junixsocket.core)
    // implementation(libs.junixsocket.native.android)
    implementation("com.kohlschutter.junixsocket:junixsocket-native-android:2.10.1@aar")
    implementation(libs.okhttp.logging.interceptor)
    implementation(libs.jnr.unixsocket)

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
