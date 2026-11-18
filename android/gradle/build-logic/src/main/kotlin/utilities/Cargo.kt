package utilities

import org.gradle.api.Project

fun Project.cargoHome(): String? =
   getStringPropertyOrNull("mullvad.app.build.cargo.home")

fun Project.cargoExecutable(): String = cargoHome()?.let { "$it/bin/cargo" } ?: "cargo"

fun Project.rustcExecutable(): String = cargoHome()?.let { "$it/bin/rustc" } ?: "rustc"
