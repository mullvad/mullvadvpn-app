package net.mullvad.mullvadvpn.test.common.misc

import net.mullvad.mullvadvpn.test.common.constant.Production
import net.mullvad.mullvadvpn.test.common.constant.Stagemole

class RelayProvider(val currentFlavor: String) {

    fun getDefaultRelay(): TestRelay {
        return when (currentFlavor) {
            "play" -> Stagemole.DEFAULT_RELAY
            "oss" -> Production.DEFAULT_RELAY
            else -> error("Invalid flavor: $currentFlavor")
        }
    }

    fun getDaitaRelay(): TestRelay {
        return when (currentFlavor) {
            "play" -> Stagemole.DAITA_RELAY
            "oss" -> Production.DAITA_RELAY
            else -> error("Invalid flavor: $currentFlavor")
        }
    }

    fun getQuicRelay(): TestRelay {
        return when (currentFlavor) {
            "play" -> Stagemole.QUIC_RELAY
            "oss" -> Production.QUIC_RELAY
            else -> error("Invalid flavor: $currentFlavor")
        }
    }

    fun getLwoRelay(): TestRelay {
        return when (currentFlavor) {
            "play" -> Stagemole.LWO_RELAY
            "oss" -> Production.LWO_RELAY
            else -> error("Invalid flavor: $currentFlavor")
        }
    }

    fun getOverrideRelay(): TestRelay {
        return when (currentFlavor) {
            "play" -> Stagemole.DEFAULT_RELAY
            "oss" -> Production.OVERRIDE_RELAY
            else -> error("Invalid flavor: $currentFlavor")
        }
    }
}

data class TestRelay(val country: String, val city: String, val relay: String)
