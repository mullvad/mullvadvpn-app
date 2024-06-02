package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState

class ApiAccessListUiStateParameterProvider : PreviewParameterProvider<ApiAccessListUiState> {

    override val values: Sequence<ApiAccessListUiState> =
        sequenceOf(
            // Default state
            ApiAccessListUiState(),
            // Without custom api access method
            ApiAccessListUiState(
                currentApiAccessMethod = defaultAccessMethods.first(),
                apiAccessMethods = defaultAccessMethods
            ),
            // With custom api
            ApiAccessListUiState(
                currentApiAccessMethod = defaultAccessMethods.first(),
                apiAccessMethods = defaultAccessMethods.plus(listOf(shadowsocks, socks5Remote))
            )
        )
}
