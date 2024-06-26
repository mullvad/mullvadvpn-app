import com.google.protobuf.gradle.proto

plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id(Dependencies.Plugin.Protobuf.protobufId) version Versions.Plugin.protobuf
    id(Dependencies.Plugin.junit5) version Versions.Plugin.junit5
}

android {
    namespace = "net.mullvad.mullvadvpn.lib.daemon.grpc"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig { minSdk = Versions.Android.minSdkVersion }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions { jvmTarget = Versions.jvmTarget }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    sourceSets {
        getByName("main") {
            proto { srcDir("${rootProject.projectDir}/../mullvad-management-interface/proto") }
        }
    }
}

protobuf {
    protoc { artifact = Dependencies.Plugin.Protobuf.protoc }
    plugins {
        create("java") { artifact = Dependencies.Plugin.Protobuf.protocGenGrpcJava }
        create("grpc") { artifact = Dependencies.Plugin.Protobuf.protocGenGrpcJava }
        create("grpckt") { artifact = Dependencies.Plugin.Protobuf.protocGenGrpcKotlin }
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
    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.talpidLib))

    implementation(Dependencies.jodaTime)
    implementation(Dependencies.kermit)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesCore)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    implementation(Dependencies.Grpc.grpcOkHttp)
    implementation(Dependencies.Grpc.grpcAndroid)
    implementation(Dependencies.Grpc.grpcKotlinStub)
    implementation(Dependencies.Grpc.protobufLite)
    implementation(Dependencies.Grpc.protobufKotlinLite)

    implementation(Dependencies.Arrow.core)
    implementation(Dependencies.Arrow.optics)

    testImplementation(project(Dependencies.Mullvad.commonTestLib))
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.turbine)
    testImplementation(Dependencies.junitApi)
    testRuntimeOnly(Dependencies.junitEngine)
    testImplementation(Dependencies.junitParams)
}
