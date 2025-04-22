package net.mullvad.mullvadvpn.compose.util

import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.SemanticsMatcher

fun withRole(role: Role): SemanticsMatcher =
    SemanticsMatcher("${SemanticsProperties.Role.name} == '$role'") {
        val roleProperty = it.config.getOrNull(SemanticsProperties.Role) ?: false
        roleProperty == role
    }
