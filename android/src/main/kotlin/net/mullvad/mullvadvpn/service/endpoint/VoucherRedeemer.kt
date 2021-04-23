package net.mullvad.mullvadvpn.service.endpoint

import net.mullvad.mullvadvpn.ipc.Event.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ipc.Request

class VoucherRedeemer(private val endpoint: ServiceEndpoint) : Actor<String>() {
    private val daemon
        get() = endpoint.intermittentDaemon

    init {
        endpoint.dispatcher.registerHandler(Request.SubmitVoucher::class) { request ->
            sendBlocking(request.voucher)
        }
    }

    fun onDestroy() = closeActor()

    override suspend fun onNewCommand(command: String) =
        endpoint.sendEvent(VoucherSubmissionResult(command, daemon.await().submitVoucher(command)))
}
