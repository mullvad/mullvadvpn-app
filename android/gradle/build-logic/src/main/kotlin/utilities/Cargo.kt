package utilities

import org.gradle.api.Project

fun Project.cargo(): String? = getStringPropertyOrNull("mullvad.app.build.cargo.command")

fun Project.cargoCommand(): String = cargo() ?: "cargo"
