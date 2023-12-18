package net.mullvad.mullvadvpn.relaylist

import io.mockk.mockk
import io.mockk.unmockkAll
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class RelayNameComparatorTest {

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_compare_respect_numbers_in_name() {
        val relay9 =
            Relay(name = "se9-wireguard", location = mockk(), locationName = "mock", active = false)
        val relay10 =
            Relay(
                name = "se10-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false
            )

        relay9 assertOrderBothDirection relay10
    }

    @Test
    fun test_compare_same_name() {
        val relay9a =
            Relay(name = "se9-wireguard", location = mockk(), locationName = "mock", active = false)
        val relay9b =
            Relay(name = "se9-wireguard", location = mockk(), locationName = "mock", active = false)

        Assertions.assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        Assertions.assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun test_compare_only_numbers_in_name() {
        val relay001 =
            Relay(name = "001", location = mockk(), locationName = "mock", active = false)
        val relay1 = Relay(name = "1", location = mockk(), locationName = "mock", active = false)
        val relay3 = Relay(name = "3", location = mockk(), locationName = "mock", active = false)
        val relay100 =
            Relay(name = "100", location = mockk(), locationName = "mock", active = false)

        relay001 assertOrderBothDirection relay1
        relay001 assertOrderBothDirection relay3
        relay1 assertOrderBothDirection relay3
        relay3 assertOrderBothDirection relay100
    }

    @Test
    fun test_compare_without_numbers_in_name() {
        val relay9a =
            Relay(name = "se-wireguard", location = mockk(), locationName = "mock", active = false)
        val relay9b =
            Relay(name = "se-wireguard", location = mockk(), locationName = "mock", active = false)

        Assertions.assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        Assertions.assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun test_compare_with_trailing_zeros_in_name() {
        val relay001 =
            Relay(
                name = "se001-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false
            )
        val relay005 =
            Relay(
                name = "se005-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false
            )

        relay001 assertOrderBothDirection relay005
    }

    @Test
    fun test_compare_prefix_and_numbers() {
        val relayAr2 =
            Relay(name = "ar2-wireguard", location = mockk(), locationName = "mock", active = false)
        val relayAr8 =
            Relay(name = "ar8-wireguard", location = mockk(), locationName = "mock", active = false)
        val relaySe5 =
            Relay(name = "se5-wireguard", location = mockk(), locationName = "mock", active = false)
        val relaySe10 =
            Relay(
                name = "se10-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false
            )

        relayAr2 assertOrderBothDirection relayAr8
        relayAr8 assertOrderBothDirection relaySe5
        relaySe5 assertOrderBothDirection relaySe10
    }

    @Test
    fun test_compare_suffix_and_numbers() {
        val relay2c =
            Relay(name = "se2-cloud", location = mockk(), locationName = "mock", active = false)
        val relay2w =
            Relay(name = "se2-wireguard", location = mockk(), locationName = "mock", active = false)

        relay2c assertOrderBothDirection relay2w
    }

    @Test
    fun test_compare_different_length() {
        val relay22a =
            Relay(name = "se22", location = mockk(), locationName = "mock", active = false)
        val relay22b =
            Relay(
                name = "se22-wireguard",
                location = mockk(),
                locationName = "mock",
                active = false
            )

        relay22a assertOrderBothDirection relay22b
    }

    private infix fun Relay.assertOrderBothDirection(other: Relay) {
        Assertions.assertTrue(RelayNameComparator.compare(this, other) < 0)
        Assertions.assertTrue(RelayNameComparator.compare(other, this) > 0)
    }
}
