package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import kotlinx.coroutines.flow.StateFlow
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.component.ShowChangelog
import net.mullvad.mullvadvpn.viewmodel.ChangelogDialogUiState

@Composable
fun ChangesListScreen(
    uiState: StateFlow<ChangelogDialogUiState>,
    onBackPressed: () -> Unit
) {
    val state = uiState.collectAsState().value
    if (state is ChangelogDialogUiState.Show) {
        ShowChangelog(
            changesList = state.changes,
            version = BuildConfig.VERSION_NAME,
            onDismiss = {
                onBackPressed()
            }
        )
    }
}
