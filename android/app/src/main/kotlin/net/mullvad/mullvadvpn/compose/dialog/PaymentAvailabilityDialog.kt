package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import net.mullvad.mullvadvpn.compose.state.PaymentState

@Composable
fun PaymentAvailabilityDialog(paymentAvailability: PaymentState, onTryAgain: () -> Unit) {
    var showPaymentAvailabilityDialog by remember { mutableStateOf(false) }

    LaunchedEffect(key1 = paymentAvailability) {
        if (paymentAvailability is PaymentState.Error) {
            showPaymentAvailabilityDialog = true
        }
    }

    if (showPaymentAvailabilityDialog) {
        PaymentBillingErrorDialog(
            onTryAgain = {
                showPaymentAvailabilityDialog = false
                onTryAgain()
            },
            onClose = { showPaymentAvailabilityDialog = false }
        )
    }
}
