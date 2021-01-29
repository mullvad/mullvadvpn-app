package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Parcelable
import kotlinx.parcelize.Parcelize

// Events that can be sent from the service
sealed class Event : Message(), Parcelable {
    protected override val messageId = 1
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    object ListenerReady : Event(), Parcelable

    companion object {
        private const val MESSAGE_KEY = "event"

        fun fromMessage(message: RawMessage): Event? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
