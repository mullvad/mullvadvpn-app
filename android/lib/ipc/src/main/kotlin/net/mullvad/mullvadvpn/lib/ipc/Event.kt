package net.mullvad.mullvadvpn.lib.ipc

import android.os.Messenger
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RemoveDeviceResult
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState

// Events that can be sent from the service
sealed class Event : Message.EventMessage() {
    override val messageKey = MESSAGE_KEY

    @Parcelize data class AccountCreationEvent(val result: AccountCreationResult) : Event()

    @Parcelize data class AccountExpiryEvent(val expiry: AccountExpiry) : Event()

    @Parcelize data class AccountHistoryEvent(val history: AccountHistory) : Event()

    @Parcelize
    data class AppVersionInfo(val versionInfo: net.mullvad.mullvadvpn.model.AppVersionInfo?) :
        Event()

    @Parcelize data class AuthToken(val token: String?) : Event()

    @Parcelize data class CurrentVersion(val version: String?) : Event()

    @Parcelize data class DeviceStateEvent(val newState: DeviceState) : Event()

    @Parcelize data class DeviceListUpdate(val event: DeviceListEvent) : Event()

    @Parcelize
    data class DeviceRemovalEvent(val deviceId: String, val result: RemoveDeviceResult) : Event()

    @Parcelize data class ListenerReady(val connection: Messenger, val listenerId: Int) : Event()

    @Parcelize data class LoginEvent(val result: LoginResult) : Event()

    @Parcelize data class NewLocation(val location: GeoIpLocation?) : Event()

    @Parcelize data class NewRelayList(val relayList: RelayList?) : Event()

    @Parcelize data class SettingsUpdate(val settings: Settings?) : Event()

    @Parcelize data class SplitTunnelingUpdate(val excludedApps: List<String>?) : Event()

    @Parcelize data class TunnelStateChange(val tunnelState: TunnelState) : Event()

    @Parcelize
    data class VoucherSubmissionResult(
        val voucher: String,
        val result: net.mullvad.mullvadvpn.model.VoucherSubmissionResult
    ) : Event()

    @Parcelize data class PlayPurchaseInitResultEvent(val result: PlayPurchaseInitResult) : Event()

    @Parcelize
    data class PlayPurchaseVerifyResultEvent(val result: PlayPurchaseVerifyResult) : Event()

    @Parcelize object VpnPermissionRequest : Event()

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: android.os.Message): Event? = fromMessage(message, MESSAGE_KEY)
    }
}

typealias EventDispatcher = MessageDispatcher<Event>
