package net.mullvad.mullvadvpn.apiaccess.impl.screen.list

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.apiaccess.impl.defaultAccessMethods
import net.mullvad.mullvadvpn.apiaccess.impl.shadowsocks
import net.mullvad.mullvadvpn.apiaccess.impl.socks5Remote

class ApiAccessListUiStatePreviewParameterProvider :
    PreviewParameterProvider<ApiAccessListUiState> {

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
