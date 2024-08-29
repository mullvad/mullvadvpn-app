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
                currentApiAccessMethodSetting = defaultAccessMethods.first(),
                apiAccessMethodSettings = defaultAccessMethods,
            ),
            // With custom api
            ApiAccessListUiState(
                currentApiAccessMethodSetting = defaultAccessMethods.first(),
                apiAccessMethodSettings =
                    defaultAccessMethods.plus(listOf(shadowsocks, socks5Remote)),
            ),
        )
}
