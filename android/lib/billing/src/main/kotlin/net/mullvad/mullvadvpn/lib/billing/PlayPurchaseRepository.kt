package net.mullvad.mullvadvpn.lib.billing

import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult

class PlayPurchaseRepository {
    suspend fun initializePlayPurchase(): PlayPurchaseInitResult {
        TODO()
        //        val result = messageHandler.trySendRequest(Request.InitPlayPurchase)
        //
        //        return if (result) {
        //            messageHandler.events<Event.PlayPurchaseInitResultEvent>().first().result
        //        } else {
        //            PlayPurchaseInitResult.Error(PlayPurchaseInitError.OtherError)
        //        }
    }

    suspend fun verifyPlayPurchase(purchase: PlayPurchase): PlayPurchaseVerifyResult {
        TODO()
        //        val result = messageHandler.trySendRequest(Request.VerifyPlayPurchase(purchase))
        //        return if (result) {
        //            messageHandler.events<Event.PlayPurchaseVerifyResultEvent>().first().result
        //        } else {
        //            PlayPurchaseVerifyResult.Error(PlayPurchaseVerifyError.OtherError)
        //        }
    }
}
