import com.google.protobuf.gradle.proto

plugins {
    id(Dependencies.Plugin.androidLibraryId)
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.kotlinParcelizeId)
    id("com.google.protobuf") version "0.9.4"
}

android {
    namespace = "net.mullvad.mullvadvpn.service"
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

    flavorDimensions += FlavorDimensions.BILLING
    flavorDimensions += FlavorDimensions.INFRASTRUCTURE

    productFlavors {
        create(Flavors.OSS) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PLAY) { dimension = FlavorDimensions.BILLING }
        create(Flavors.PROD) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            isDefault = true
            // Not used for production builds.
            buildConfigField("String", "API_ENDPOINT", "\"\"")
        }
        create(Flavors.DEVMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.devmole.eu\"")
        }
        create(Flavors.STAGEMOLE) {
            dimension = FlavorDimensions.INFRASTRUCTURE
            buildConfigField("String", "API_ENDPOINT", "\"api.stagemole.eu\"")
        }
    }

    sourceSets {
        getByName("main") {
            proto { srcDir("${rootProject.projectDir}/../mullvad-management-interface/proto") }
        }
    }
    packagingOptions {
        this.excludes.add("META-INF/*")
        resources {
            excludes.add("META-INF/*")
        }
    }
    packaging { resources { excludes.add("META-INF/*") } }

    buildFeatures {
        buildConfig = true
    }
}

protobuf {
    // Configure the protoc executable
    protoc {
        this.artifact = "com.google.protobuf:protoc:3.15.0"
        // Download from repositories
        // artifact("com.google.protobuf:protoc:3.0.0")
    }
    plugins {
        create("java") { artifact = "io.grpc:protoc-gen-grpc-java:1.57.2" }
        create("grpc") { artifact = "io.grpc:protoc-gen-grpc-java:1.57.2" }
        create("kotlin") { artifact = "io.grpc:protoc-gen-grpc-kotlin:1.4.0:jdk8@jar" }
    }
    generateProtoTasks {
        all().forEach {
            it.plugins {
                create("java") { option("lite") }
                create("grpc") { option("lite") }
                create("kotlin") { option("lite") }
            }
            /*it.builtins {
                create("kotlin") {
                    option("lite")
                }
            }*/
        }
    }
}

dependencies {
    val grpcVersion = "1.57.2"
    val grpcKotlinVersion = "1.4.0" // CURRENT_GRPC_KOTLIN_VERSION
    val protobufVersion = "3.24.1"
    val coroutinesVersion = "1.7.3"

    implementation(project(Dependencies.Mullvad.commonLib))
    implementation(project(Dependencies.Mullvad.endpointLib))
    implementation(project(Dependencies.Mullvad.ipcLib))
    implementation(project(Dependencies.Mullvad.modelLib))
    implementation(project(Dependencies.Mullvad.talpidLib))

    implementation(Dependencies.jodaTime)
    implementation(Dependencies.Koin.core)
    implementation(Dependencies.Koin.android)
    implementation(Dependencies.Kotlin.stdlib)
    implementation(Dependencies.KotlinX.coroutinesAndroid)

    // implementation("io.netty:netty-all:4.1.0.CR1")
    implementation("io.grpc:grpc-netty:1.57.2")
    api("io.grpc:grpc-stub:1.57.2")
    api("io.grpc:grpc-protobuf:1.57.2")
    api("com.google.protobuf:protobuf-java-util:3.24.1")
    api("com.google.protobuf:protobuf-kotlin:3.24.1")
    api("io.grpc:grpc-kotlin-stub:1.4.0")
    // api("com.google.protobuf:protobuf-kotlin-lite:$protobufVersion")
}
