package net.mullvad.mullvadvpn.test.e2e.misc

import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.constant.Production
import net.mullvad.mullvadvpn.test.e2e.constant.Stagemole

class RelayProvider(val currentFlavor: String = BuildConfig.FLAVOR_billing) {

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
}

data class TestRelay(val country: String, val city: String, val relay: String)
