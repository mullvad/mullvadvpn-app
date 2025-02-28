package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.tooling.preview.PreviewScreenSizes
import net.mullvad.mullvadvpn.compose.preview.AccountUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.AccountUiState

class AccountScreenshots {

    @OptIn(ExperimentalMaterial3Api::class)
    @Preview("AccountScreen")
    @PreviewScreenSizes
    @Composable
    private fun PreviewAccountScreen(
        @PreviewParameter(AccountUiStatePreviewParameterProvider::class) state: AccountUiState
    ) {
        AppTheme {
            AccountScreen(state = state, SnackbarHostState(), {}, {}, {}, {}, {}, {}, {}, {})
        }
    }
}
