package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.component.ShowChangesDialog
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogDialogState

@Composable
fun ChangesListScreen(
    viewModel: AppChangesViewModel,
    onBackPressed: () -> Unit
) {
    val state = viewModel.changeLogUiState.collectAsState().value
    if (state is ChangelogDialogState.Show) {
        ShowChangesDialog(
            changesList = state.changelogList,
            version = BuildConfig.VERSION_NAME,
            onDismiss = {
                onBackPressed()
            }
        )
    }
}
