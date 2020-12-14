package net.mullvad.mullvadvpn.service

import android.os.Bundle
import android.os.Message
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.Settings

sealed class Event {
    abstract val type: Type

    val message: Message
        get() = Message.obtain().apply {
            what = type.ordinal
            data = Bundle()

            prepareData(data)
        }

    open fun prepareData(data: Bundle) {}

    class NewLocation(val location: GeoIpLocation?) : Event() {
        companion object {
            private val locationKey = "location"
        }

        override val type = Type.NewLocation

        constructor(data: Bundle) : this(data.getParcelable(locationKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(locationKey, location)
        }
    }

    class SettingsUpdate(val settings: Settings?) : Event() {
        companion object {
            private val settingsKey = "settings"
        }

        override val type = Type.SettingsUpdate

        constructor(data: Bundle) : this(data.getParcelable(settingsKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(settingsKey, settings)
        }
    }

    class WireGuardKeyStatus(val keyStatus: KeygenEvent?) : Event() {
        companion object {
            private val keyStatusKey = "keyStatus"
        }

        override val type = Type.WireGuardKeyStatus

        constructor(data: Bundle) : this(data.getParcelable(keyStatusKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(keyStatusKey, keyStatus)
        }
    }

    enum class Type(val build: (Bundle) -> Event) {
        NewLocation({ data -> NewLocation(data) }),
        SettingsUpdate({ data -> SettingsUpdate(data) }),
        WireGuardKeyStatus({ data -> WireGuardKeyStatus(data) }),
    }

    companion object {
        fun fromMessage(message: Message): Event {
            val type = Type.values()[message.what]

            return type.build(message.data)
        }
    }
}
