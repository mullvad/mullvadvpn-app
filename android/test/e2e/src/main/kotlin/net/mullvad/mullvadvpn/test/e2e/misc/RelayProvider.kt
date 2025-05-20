package net.mullvad.mullvadvpn.test.e2e.misc

import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.constant.Production
import net.mullvad.mullvadvpn.test.e2e.constant.Stagemole

class RelayProvider(val currentFlavor: String = BuildConfig.FLAVOR_billing) {

    fun getDefaultRelay(): TestRelay {
        return when (currentFlavor) {
            "play" ->
                TestRelay(
                    country = Stagemole.DEFAULT_COUNTRY,
                    city = Stagemole.DEFAULT_CITY,
                    relay = Stagemole.DEFAULT_RELAY,
                )
            "oss" ->
                TestRelay(
                    country = Production.DEFAULT_COUNTRY,
                    city = Production.DEFAULT_CITY,
                    relay = Production.DEFAULT_RELAY,
                )
            else -> error("Invalid flavor: $currentFlavor")
        }
    }

    fun getDaitaRelay(): TestRelay {
        return when (currentFlavor) {
            "play" ->
                TestRelay(
                    country = Stagemole.DAITA_COMPATIBLE_COUNTRY,
                    city = Stagemole.DAITA_COMPATIBLE_CITY,
                    relay = Stagemole.DAITA_COMPATIBLE_RELAY,
                )
            "oss" ->
                TestRelay(
                    country = Production.DAITA_COMPATIBLE_COUNTRY,
                    city = Production.DAITA_COMPATIBLE_CITY,
                    relay = Production.DAITA_COMPATIBLE_RELAY,
                )
            else -> error("Invalid flavor: $currentFlavor")
        }
    }
}

data class TestRelay(val country: String, val city: String, val relay: String)
