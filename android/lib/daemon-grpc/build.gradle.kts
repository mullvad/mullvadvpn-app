import com.google.protobuf.gradle.proto

plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id("com.google.protobuf") version "0.9.4"
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
        baseline = file("lint-baseline.xml")
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
    protoc { artifact = "com.google.protobuf:protoc:3.24.1" }
    plugins {
        create("java") { artifact = "io.grpc:protoc-gen-grpc-java:1.57.2" }
        create("grpc") { artifact = "io.grpc:protoc-gen-grpc-java:1.57.2" }
        create("grpckt") { artifact = "io.grpc:protoc-gen-grpc-kotlin:1.4.0:jdk8@jar" }
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
    val grpcVersion = "1.57.2"
    val grpcKotlinVersion = "1.4.0" // CURRENT_GRPC_KOTLIN_VERSION
    val protobufVersion = "3.24.1"
    val coroutinesVersion = "1.7.3"
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.commonLib))

    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    implementation("io.grpc:grpc-okhttp:$grpcVersion")
    //    implementation("io.grpc:grpc-netty:1.57.2")
    //    api("io.grpc:grpc-stub:$grpcVersion")
    implementation("io.grpc:grpc-stub:$grpcVersion")
    implementation("io.grpc:grpc-android:$grpcVersion")
    implementation("io.grpc:grpc-kotlin-stub:$grpcKotlinVersion")
    //    api("io.grpc:grpc-protobuf:$grpcVersion")
    implementation("io.grpc:grpc-protobuf-lite:$grpcVersion")
    implementation("com.google.protobuf:protobuf-kotlin-lite:$protobufVersion")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:$coroutinesVersion")

    //    api("com.google.protobuf:protobuf-java-util:$protobufVersion")
    //    api("com.google.protobuf:protobuf-kotlin:$protobufVersion")
}
