package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request

class VoucherRedeemer(val connection: Messenger, eventDispatcher: EventDispatcher) {
    private val activeSubmissions =
        mutableMapOf<String, CompletableDeferred<VoucherSubmissionResult>>()

    init {
        eventDispatcher.registerHandler(
            Event.Type.VoucherSubmissionResult
        ) { event: Event.VoucherSubmissionResult ->
            activeSubmissions.remove(event.voucher)?.complete(event.result)
        }
    }

    suspend fun submit(voucher: String): VoucherSubmissionResult {
        val result = CompletableDeferred<VoucherSubmissionResult>()

        activeSubmissions.put(voucher, result)

        connection.send(Request.SubmitVoucher(voucher).message)

        return result.await()
    }

    fun onDestroy() {
        for ((_, submission) in activeSubmissions) {
            submission.cancel()
        }

        activeSubmissions.clear()
    }
}
