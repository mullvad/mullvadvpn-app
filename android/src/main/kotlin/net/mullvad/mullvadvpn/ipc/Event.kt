package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.Settings

// Events that can be sent from the service
sealed class Event : Message(), Parcelable {
    protected override val messageId = 1
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    object ListenerReady : Event(), Parcelable

    @Parcelize
    data class NewLocation(val location: GeoIpLocation?) : Event(), Parcelable

    @Parcelize
    data class SettingsUpdate(val settings: Settings?) : Event(), Parcelable

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: RawMessage): Event? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
