import com.google.protobuf.gradle.proto

plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id(Dependencies.Plugin.protobufId) version Versions.Plugin.protobuf
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
    protoc { artifact = "com.google.protobuf:protoc:${Versions.Grpc.protobufVersion}" }
    plugins {
        create("java") { artifact = "io.grpc:protoc-gen-grpc-java:${Versions.Grpc.grpcVersion}" }
        create("grpc") { artifact = "io.grpc:protoc-gen-grpc-java:${Versions.Grpc.grpcVersion}" }
        create("grpckt") {
            artifact = "io.grpc:protoc-gen-grpc-kotlin:${Versions.Grpc.grpcKotlinVersion}:jdk8@jar"
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
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.talpidLib))

    implementation(Dependencies.jodaTime)
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
    implementation(Dependencies.Arrow.fxCoroutines)

    testImplementation(project(Dependencies.Mullvad.commonTestLib))
    testImplementation(Dependencies.Kotlin.test)
    testImplementation(Dependencies.KotlinX.coroutinesTest)
    testImplementation(Dependencies.MockK.core)
    testImplementation(Dependencies.turbine)
    testImplementation(Dependencies.junitApi)
    testRuntimeOnly(Dependencies.junitEngine)
    testImplementation(Dependencies.junitParams)
}
