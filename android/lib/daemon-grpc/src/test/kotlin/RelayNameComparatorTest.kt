package net.mullvad.mullvadvpn.lib.daemon.grpc

import io.mockk.mockk
import io.mockk.unmockkAll
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.ProviderId
import net.mullvad.mullvadvpn.model.RelayItem
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
                        ownership = Ownership.MullvadOwned
                    ),
            )
        val relay10 =
            RelayItem.Location.Relay(
                name = "se10-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        relay9 assertOrderBothDirection relay10
    }

    @Test
    fun `given two relays with same name with number in name comparator should return 0`() {
        val relay9a =
            RelayItem.Location.Relay(
                name = "se9-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay9b =
            RelayItem.Location.Relay(
                name = "se9-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun `comparator should be able to handle name of only numbers`() {
        val relay001 =
            RelayItem.Location.Relay(
                name = "001",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay1 =
            RelayItem.Location.Relay(
                name = "1",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay3 =
            RelayItem.Location.Relay(
                name = "3",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay100 =
            RelayItem.Location.Relay(
                name = "100",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
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
                name = "se-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay9b =
            RelayItem.Location.Relay(
                name = "se-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun `given two relays with leading zeroes comparator should return lowest number first`() {
        val relay001 =
            RelayItem.Location.Relay(
                name = "se001-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay005 =
            RelayItem.Location.Relay(
                name = "se005-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        relay001 assertOrderBothDirection relay005
    }

    @Test
    fun `given 4 relays comparator should sort by prefix then number`() {
        val relayAr2 =
            RelayItem.Location.Relay(
                name = "ar2-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relayAr8 =
            RelayItem.Location.Relay(
                name = "ar8-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relaySe5 =
            RelayItem.Location.Relay(
                name = "se5-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relaySe10 =
            RelayItem.Location.Relay(
                name = "se10-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        relayAr2 assertOrderBothDirection relayAr8
        relayAr8 assertOrderBothDirection relaySe5
        relaySe5 assertOrderBothDirection relaySe10
    }

    @Test
    fun `given two relays with same prefix and number comparator should sort by suffix`() {
        val relay2c =
            RelayItem.Location.Relay(
                name = "se2-cloud",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay2w =
            RelayItem.Location.Relay(
                name = "se2-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )

        relay2c assertOrderBothDirection relay2w
    }

    @Test
    fun `given two relays with same prefix, but one with no suffix, the one with no suffix should come first`() {
        val relay22a =
            RelayItem.Location.Relay(
                name = "se22",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
            )
        val relay22b =
            RelayItem.Location.Relay(
                name = "se22-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false,
                providerName = "Provider",
                ownership = Ownership.MullvadOwned
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
