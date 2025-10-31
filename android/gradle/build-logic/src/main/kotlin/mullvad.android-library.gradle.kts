import com.android.build.gradle.LibraryExtension
import org.gradle.api.JavaVersion
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.getByType
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension

apply(plugin = "com.android.library")
apply(plugin = "org.jetbrains.kotlin.android")

val libs = extensions.getByType<VersionCatalogsExtension>().named("libs")

configure<LibraryExtension> {
    compileSdk = libs.findVersion("compile-sdk").get().toString().toInt()
    buildToolsVersion = libs.findVersion("build-tools").get().toString()

    defaultConfig { minSdk = libs.findVersion("min-sdk").get().toString().toInt() }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    lint {
        lintConfig = file("${project.rootDir}/config/lint.xml")
        abortOnError = true
        warningsAsErrors = true
    }
}

configure<KotlinAndroidProjectExtension> {
    compilerOptions {
        jvmTarget = JvmTarget.fromTarget(libs.findVersion("jvm-target").get().toString())
        allWarningsAsErrors = true
    }
}
