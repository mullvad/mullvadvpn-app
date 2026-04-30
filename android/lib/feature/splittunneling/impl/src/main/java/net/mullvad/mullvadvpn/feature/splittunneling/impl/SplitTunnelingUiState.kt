package net.mullvad.mullvadvpn.feature.splittunneling.impl

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.AnnotatedString
import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.ui.component.highlightText
import net.mullvad.mullvadvpn.lib.ui.theme.color.highlight

data class Loading(val isModal: Boolean = false)

data class SplitTunnelingUiState(
    val enabled: Boolean = false,
    val excludedApps: List<AppItem> = emptyList(),
    val includedApps: List<AppItem> = emptyList(),
    val showSystemApps: Boolean = false,
    val isModal: Boolean = false,
)

data class AppItem(val title: String, val packageName: PackageName, private val highlight: String = "") {
    val titleAnnotated: AnnotatedString
        @Composable get() = title.highlightText(highlight, MaterialTheme.colorScheme.highlight)
}
