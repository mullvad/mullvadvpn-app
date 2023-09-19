package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import net.mullvad.mullvadvpn.compose.state.PaymentState

@Composable
fun PaymentAvailabilityErrorDialog(paymentAvailability: PaymentState, retry: () -> Unit) {
    var showPaymentAvailabilityDialog by remember(paymentAvailability) {
        mutableStateOf(paymentAvailability is PaymentState.Error)
    }

    if (showPaymentAvailabilityDialog) {
        PaymentBillingErrorDialog(
            retry = {
                showPaymentAvailabilityDialog = false
                retry()
            },
            onClose = { showPaymentAvailabilityDialog = false }
        )
    }
}
