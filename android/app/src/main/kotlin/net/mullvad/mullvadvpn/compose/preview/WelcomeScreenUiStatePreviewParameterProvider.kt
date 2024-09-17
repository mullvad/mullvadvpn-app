package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice

class WelcomeScreenUiStatePreviewParameterProvider : PreviewParameterProvider<WelcomeUiState> {
    override val values =
        sequenceOf(
            WelcomeUiState(
                tunnelState = TunnelStatePreviewData.generateDisconnectedState(),
                accountNumber = AccountNumber("4444555566667777"),
                deviceName = "Happy Mole",
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(PaymentProduct(ProductId("product"), ProductPrice("$44"), null))
                    ),
            )
        )
}
