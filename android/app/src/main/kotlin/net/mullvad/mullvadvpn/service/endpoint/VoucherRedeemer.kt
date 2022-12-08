package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request

class VoucherRedeemer(private val endpoint: ServiceEndpoint) {
    private val daemon
        get() = endpoint.intermittentDaemon

    private val voucherChannel = spawnActor()

    init {
        endpoint.dispatcher.registerHandler(Request.SubmitVoucher::class) { request ->
            voucherChannel.trySendBlocking(request.voucher)
        }
    }

    fun onDestroy() {
        voucherChannel.close()
    }

    private fun spawnActor() = GlobalScope.actor<String>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            for (voucher in channel) {
                val result = daemon.await().submitVoucher(voucher)

                endpoint.sendEvent(Event.VoucherSubmissionResult(voucher, result))
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Voucher channel was closed, stop the actor
        }
    }
}
