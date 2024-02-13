import com.android.build.gradle.internal.tasks.factory.dependsOn

plugins {
    id(Dependencies.Plugin.kotlinAndroidId)
    id(Dependencies.Plugin.androidLibraryId)
}

val repoRootPath = rootProject.projectDir.absoluteFile.parentFile.absolutePath
val mapResourcesRaw = "$repoRootPath/android/lib/map/src/main/res/raw/"

android {
    namespace = "net.mullvad.mullvadvpn.lib.map"
    compileSdk = Versions.Android.compileSdkVersion

    defaultConfig {
        minSdk = Versions.Android.minSdkVersion
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = Versions.jvmTarget
    }

    buildFeatures { compose = true }

    composeOptions { kotlinCompilerExtensionVersion = Versions.kotlinCompilerExtensionVersion }

    lint {
        lintConfig = file("${rootProject.projectDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }

    project.tasks.preBuild.dependsOn("copyMapData")
}

tasks.register("copyMapData", Copy::class) {
    from("$repoRootPath/gui/assets/geo")
    include("land_contour_indices.bin")
    include("land_positions.bin")
    include("land_triangle_indices.bin")
    include("ocean_indices.bin")
    include("ocean_positions.bin")
    into(mapResourcesRaw)
}

dependencies {

    //Model
    implementation(project(Dependencies.Mullvad.modelLib))

    implementation(Dependencies.Compose.ui)
    implementation(Dependencies.Compose.foundation)

    implementation(Dependencies.AndroidX.lifecycleRuntimeKtx)
}