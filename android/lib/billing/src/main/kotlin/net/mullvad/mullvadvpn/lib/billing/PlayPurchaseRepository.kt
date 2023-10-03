package net.mullvad.mullvadvpn.lib.billing

import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult

class PlayPurchaseRepository(private val messageHandler: MessageHandler) {
    suspend fun purchaseInitialisation(): PlayPurchaseInitResult {
        val result = messageHandler.trySendRequest(Request.InitPlayPurchase)

        return if (result) {
            messageHandler.events<Event.PlayPurchaseInitResultEvent>().first().result
        } else {
            PlayPurchaseInitResult.Error(PlayPurchaseInitError.OtherError)
        }
    }

    suspend fun purchaseVerification(purchase: PlayPurchase): PlayPurchaseVerifyResult {
        val result = messageHandler.trySendRequest(Request.VerifyPlayPurchase(purchase))
        return if (result) {
            messageHandler.events<Event.PlayPurchaseVerifyResultEvent>().first().result
        } else {
            PlayPurchaseVerifyResult.Error(PlayPurchaseVerifyError.OtherError)
        }
    }
}
