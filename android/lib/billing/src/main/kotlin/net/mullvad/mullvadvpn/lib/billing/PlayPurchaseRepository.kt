package net.mullvad.mullvadvpn.lib.billing

import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.PlayPurchase

class PlayPurchaseRepository(private val managementService: ManagementService) {
    suspend fun initializePlayPurchase() = managementService.initializePlayPurchase()

    suspend fun verifyPlayPurchase(purchase: PlayPurchase) =
        managementService.verifyPlayPurchase(purchase)
}
