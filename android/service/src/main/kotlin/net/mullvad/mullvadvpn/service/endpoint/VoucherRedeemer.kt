package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.lib.common.util.parseAsDateTime
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult

class VoucherRedeemer(
    private val endpoint: ServiceEndpoint,
    private val accountCache: AccountCache
) {
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

    private fun spawnActor() =
        GlobalScope.actor<String>(Dispatchers.Default, Channel.UNLIMITED) {
            try {
                for (voucher in channel) {
                    val result = daemon.await().submitVoucher(voucher)

                    // Let AccountCache know about the new expiry
                    if (result is VoucherSubmissionResult.Ok) {
                        val accountExpiry =
                            AccountExpiry.Available(result.submission.newExpiry.parseAsDateTime()!!)
                        accountCache.onAccountExpiryChange.notify(accountExpiry)
                    }
                    endpoint.sendEvent(Event.VoucherSubmissionResult(voucher, result))
                }
            } catch (exception: ClosedReceiveChannelException) {
                // Voucher channel was closed, stop the actor
            }
        }
}
