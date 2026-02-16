package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.list

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.feature.apiaccess.impl.defaultAccessMethods
import net.mullvad.mullvadvpn.feature.apiaccess.impl.shadowsocks
import net.mullvad.mullvadvpn.feature.apiaccess.impl.socks5Remote

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
