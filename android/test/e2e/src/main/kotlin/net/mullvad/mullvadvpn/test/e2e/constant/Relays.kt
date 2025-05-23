package net.mullvad.mullvadvpn.test.e2e.constant

import net.mullvad.mullvadvpn.test.e2e.misc.TestRelay

object Stagemole {
    val DEFAULT_RELAY = Relays.gotWg001
    val DAITA_RELAY = Relays.gotWg002RelaySoftware
}

object Production {
    val DEFAULT_RELAY = Relays.gotWg001
    val DAITA_RELAY = Relays.gotWg002
}

private object Relays {
    val gotWg001 = TestRelay(relay = "se-got-wg-001", country = "Sweden", city = "Gothenburg")
    val gotWg002 = TestRelay(relay = "se-got-wg-002", country = "Sweden", city = "Gothenburg")
    val gotWg002RelaySoftware =
        TestRelay(
            relay = "se-got-wg-002",
            country = "Relay Software Country",
            city = "Relay Software City",
        )
}
