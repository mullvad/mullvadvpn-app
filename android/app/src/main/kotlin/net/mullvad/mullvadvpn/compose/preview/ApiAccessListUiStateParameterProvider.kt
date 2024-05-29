package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.Port

class ApiAccessListUiStateParameterProvider : PreviewParameterProvider<ApiAccessListUiState> {
    private val defaultAccessMethods =
        listOf(
            ApiAccessMethod(
                ApiAccessMethodId.fromString(UUID1),
                ApiAccessMethodName("Direct"),
                enabled = true,
                ApiAccessMethodType.Direct
            ),
            ApiAccessMethod(
                ApiAccessMethodId.fromString(UUID2),
                ApiAccessMethodName("Bridges"),
                enabled = false,
                ApiAccessMethodType.Bridges
            )
        )

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
                apiAccessMethods =
                    defaultAccessMethods.plus(
                        listOf(
                            ApiAccessMethod(
                                ApiAccessMethodId.fromString(UUID3),
                                ApiAccessMethodName("Custom"),
                                enabled = true,
                                ApiAccessMethodType.CustomProxy.Shadowsocks(
                                    ip = "192.168.1.1",
                                    port = Port(123),
                                    password = "Password",
                                    cipher = "Cipher"
                                )
                            )
                        )
                    )
            )
        )

    companion object {
        private const val UUID1 = "12345678-1234-5678-1234-567812345678"
        private const val UUID2 = "12345678-1234-5678-1234-567812345679"
        private const val UUID3 = "12345678-1234-5678-1234-567812345671"
    }
}
