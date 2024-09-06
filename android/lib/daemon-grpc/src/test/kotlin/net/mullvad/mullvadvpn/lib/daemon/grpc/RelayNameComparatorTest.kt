package net.mullvad.mullvadvpn.lib.daemon.grpc

import io.mockk.mockk
import io.mockk.unmockkAll
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class RelayNameComparatorTest {

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `given two relays with same prefix but different numbers comparator should return lowest number first`() {
        val relay9 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se9-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay10 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se10-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relay9 assertOrderBothDirection relay10
    }

    @Test
    fun `given two relays with same name with number in name comparator should return 0`() {
        val relay9a =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se9-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay9b =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se9-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun `comparator should be able to handle name of only numbers`() {
        val relay001 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "001"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay1 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "1"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay3 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "3"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay100 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "100"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relay001 assertOrderBothDirection relay1
        relay001 assertOrderBothDirection relay3
        relay1 assertOrderBothDirection relay3
        relay3 assertOrderBothDirection relay100
    }

    @Test
    fun `given two relays with same name and without number comparator should return 0`() {
        val relay9a =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay9b =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun `given two relays with leading zeroes comparator should return lowest number first`() {
        val relay001 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se001-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay005 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se005-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relay001 assertOrderBothDirection relay005
    }

    @Test
    fun `given 4 relays comparator should sort by prefix then number`() {
        val relayAr2 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "ar2-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relayAr8 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "ar8-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relaySe5 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se5-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relaySe10 =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se10-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relayAr2 assertOrderBothDirection relayAr8
        relayAr8 assertOrderBothDirection relaySe5
        relaySe5 assertOrderBothDirection relaySe10
    }

    @Test
    fun `given two relays with same prefix and number comparator should sort by suffix`() {
        val relay2c =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se2-cloud"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay2w =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se2-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relay2c assertOrderBothDirection relay2w
    }

    @Test
    fun `given two relays with same prefix, but one with no suffix, the one with no suffix should come first`() {
        val relay22a =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se22"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )
        val relay22b =
            RelayItem.Location.Relay(
                id = GeoLocationId.Hostname(city = mockk(), "se22-wireguard"),
                active = false,
                provider =
                    Provider(
                        providerId = ProviderId("Provider"),
                        ownership = Ownership.MullvadOwned,
                    ),
                daita = false,
            )

        relay22a assertOrderBothDirection relay22b
    }

    private infix fun RelayItem.Location.Relay.assertOrderBothDirection(
        other: RelayItem.Location.Relay
    ) {
        assertTrue(RelayNameComparator.compare(this, other) < 0)
        assertTrue(RelayNameComparator.compare(other, this) > 0)
    }
}
