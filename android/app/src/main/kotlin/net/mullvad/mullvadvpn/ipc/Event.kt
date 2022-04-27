package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AppVersionInfo as AppVersionInfoData
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LoginStatus as LoginStatusData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult as VoucherSubmissionResultData

// Events that can be sent from the service
sealed class Event : Message.EventMessage() {
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    data class AccountCreationEvent(val result: AccountCreationResult) : Event()

    @Parcelize
    data class AccountHistory(val history: String?) : Event()

    @Parcelize
    data class AppVersionInfo(val versionInfo: AppVersionInfoData?) : Event()

    @Parcelize
    data class AuthToken(val token: String?) : Event()

    @Parcelize
    data class CurrentVersion(val version: String?) : Event()

    @Parcelize
    data class DeviceStateEvent(val newState: DeviceState) : Event()

    @Parcelize
    data class ListenerReady(val connection: Messenger, val listenerId: Int) : Event()

    @Parcelize
    data class LoginStatus(val status: LoginStatusData?) : Event()

    @Parcelize
    data class NewLocation(val location: GeoIpLocation?) : Event()

    @Parcelize
    data class NewRelayList(val relayList: RelayList?) : Event()

    @Parcelize
    data class SettingsUpdate(val settings: Settings?) : Event()

    @Parcelize
    data class SplitTunnelingUpdate(val excludedApps: List<String>?) : Event()

    @Parcelize
    data class TunnelStateChange(val tunnelState: TunnelState) : Event()

    @Parcelize
    data class VoucherSubmissionResult(
        val voucher: String,
        val result: VoucherSubmissionResultData
    ) : Event()

    @Parcelize
    object VpnPermissionRequest : Event()

    @Parcelize
    data class WireGuardKeyStatus(val keyStatus: KeygenEvent?) : Event()

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: RawMessage): Event? = Message.fromMessage(message, MESSAGE_KEY)
    }
}

typealias EventDispatcher = MessageDispatcher<Event>
