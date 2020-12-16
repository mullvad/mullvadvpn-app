package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LoginStatus as LoginStatusData
import net.mullvad.mullvadvpn.model.Settings

// Events that can be sent from the service
sealed class Event : Message.EventMessage() {
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    data class AccountHistory(val history: List<String>?) : Event()

    @Parcelize
    object ListenerReady : Event()

    @Parcelize
    data class LoginStatus(val status: LoginStatusData?) : Event()

    @Parcelize
    data class NewLocation(val location: GeoIpLocation?) : Event()

    @Parcelize
    data class SettingsUpdate(val settings: Settings?) : Event()

    @Parcelize
    data class WireGuardKeyStatus(val keyStatus: KeygenEvent?) : Event()

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: RawMessage): Event? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
