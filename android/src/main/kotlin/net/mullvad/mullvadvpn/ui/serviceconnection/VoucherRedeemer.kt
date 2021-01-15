package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.MessageDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult

class VoucherRedeemer(val connection: Messenger, eventDispatcher: MessageDispatcher<Event>) {
    private val activeSubmissions =
        mutableMapOf<String, CompletableDeferred<VoucherSubmissionResult>>()

    init {
        eventDispatcher.registerHandler(Event.VoucherSubmissionResult::class) { event ->
            synchronized(this@VoucherRedeemer) {
                activeSubmissions.remove(event.voucher)?.complete(event.result)
            }
        }
    }

    suspend fun submit(voucher: String): VoucherSubmissionResult {
        val result = CompletableDeferred<VoucherSubmissionResult>()

        synchronized(this) {
            activeSubmissions.put(voucher, result)
        }

        connection.send(Request.SubmitVoucher(voucher).message)

        return result.await()
    }

    fun onDestroy() {
        synchronized(this) {
            for ((_, submission) in activeSubmissions) {
                submission.cancel()
            }

            activeSubmissions.clear()
        }
    }
}
