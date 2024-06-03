package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState

class ApiAccessMethodDetailsUiStatePreviewParameterProvider :
    PreviewParameterProvider<ApiAccessMethodDetailsUiState> {
    override val values: Sequence<ApiAccessMethodDetailsUiState> =
        sequenceOf(
            ApiAccessMethodDetailsUiState.Loading,
            // Non-editable api access type
            defaultAccessMethods[0].let {
                ApiAccessMethodDetailsUiState.Content(
                    it.name,
                    enabled = it.enabled,
                    canBeEdited = false,
                    currentMethod = false,
                    canBeDisabled = true,
                    testApiAccessMethodState = null
                )
            },
            // Editable api access type, current method, can not be disabled
            shadowsocks.let {
                ApiAccessMethodDetailsUiState.Content(
                    it.name,
                    enabled = it.enabled,
                    canBeEdited = true,
                    currentMethod = true,
                    canBeDisabled = false,
                    testApiAccessMethodState = null
                )
            }
        )
}
