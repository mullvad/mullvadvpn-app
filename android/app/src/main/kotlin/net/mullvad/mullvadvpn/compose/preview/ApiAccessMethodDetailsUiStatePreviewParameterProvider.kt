package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState

class ApiAccessMethodDetailsUiStatePreviewParameterProvider :
    PreviewParameterProvider<ApiAccessMethodDetailsUiState> {
    override val values: Sequence<ApiAccessMethodDetailsUiState> =
        sequenceOf(
            ApiAccessMethodDetailsUiState.Loading(shadowsocks.id),
            // Non-editable api access type
            defaultAccessMethods[0].let {
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodId = it.id,
                    name = it.name,
                    enabled = it.enabled,
                    isEditable = false,
                    isCurrentMethod = false,
                    isDisableable = true,
                    isTestingAccessMethod = false,
                )
            },
            // Editable api access type, current method, can not be disabled
            shadowsocks.let {
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodId = it.id,
                    name = it.name,
                    enabled = it.enabled,
                    isEditable = true,
                    isCurrentMethod = true,
                    isDisableable = false,
                    isTestingAccessMethod = false,
                )
            },
        )
}
