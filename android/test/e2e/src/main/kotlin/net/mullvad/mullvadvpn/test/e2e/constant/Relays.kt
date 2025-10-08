package net.mullvad.mullvadvpn.test.e2e.constant

import net.mullvad.mullvadvpn.test.e2e.misc.TestRelay

object Stagemole {
    val DEFAULT_RELAY = Relays.gotWg001
    val DAITA_RELAY = Relays.gotWg002RelaySoftware
    val QUIC_RELAY = Relays.stoWg001
    val LWO_RELAY = Relays.stoWg001
}

object Production {
    val DEFAULT_RELAY = Relays.gotWg001
    val DAITA_RELAY = Relays.gotWg002
    val QUIC_RELAY = Relays.stoWg204
    val LWO_RELAY = Relays.stoWg204
}

private object Relays {
    val gotWg001 = TestRelay(relay = "se-got-wg-001", country = "Sweden", city = "Gothenburg")
    val gotWg002 = TestRelay(relay = "se-got-wg-002", country = "Sweden", city = "Gothenburg")

    val stoWg001 = TestRelay(relay = "se-sto-wg-001", country = "Sweden", city = "Stockholm")
    val stoWg204 = TestRelay(relay = "se-sto-wg-204", country = "Sweden", city = "Stockholm")
    val gotWg002RelaySoftware =
        TestRelay(
            relay = "se-got-wg-002",
            country = "Relay Software Country",
            city = "Relay Software city",
        )
}
