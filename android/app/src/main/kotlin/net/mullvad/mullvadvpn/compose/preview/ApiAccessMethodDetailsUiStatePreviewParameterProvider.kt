package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.Port

class ApiAccessMethodDetailsUiStatePreviewParameterProvider :
    PreviewParameterProvider<ApiAccessMethodDetailsUiState> {
    override val values: Sequence<ApiAccessMethodDetailsUiState> =
        sequenceOf(
            ApiAccessMethodDetailsUiState.Loading(shadowsocks.id),
            // Non-editable api access type
            defaultAccessMethods[0].let {
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodSetting =
                        ApiAccessMethodSetting(
                            id = it.id,
                            name = it.name,
                            enabled = it.enabled,
                            apiAccessMethod = ApiAccessMethod.Direct,
                        ),
                    isCurrentMethod = false,
                    isDisableable = true,
                    isTestingAccessMethod = false,
                )
            },
            // Editable api access type, current method, can not be disabled
            shadowsocks.let {
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodSetting =
                        ApiAccessMethodSetting(
                            id = it.id,
                            name = it.name,
                            enabled = it.enabled,
                            apiAccessMethod =
                                ApiAccessMethod.CustomProxy.Shadowsocks(
                                    "123.123.123.123",
                                    Port.fromString("1234").getOrNull()!!,
                                    null,
                                    Cipher.CHACHA20_IETF_POLY1305,
                                ),
                        ),
                    isCurrentMethod = true,
                    isDisableable = false,
                    isTestingAccessMethod = false,
                )
            },
        )
}
