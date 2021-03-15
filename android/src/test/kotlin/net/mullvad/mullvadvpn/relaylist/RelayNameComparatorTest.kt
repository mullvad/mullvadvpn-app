package net.mullvad.mullvadvpn.relaylist

import io.mockk.mockk
import io.mockk.unmockkAll
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Test

class RelayNameComparatorTest {

    private val mockedCity = mockk<RelayCity>(relaxed = true)

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_compare_respect_numbers_in_name() {
        val relay9 = Relay(mockedCity, "se9-wireguard", false)
        val relay10 = Relay(mockedCity, "se10-wireguard", false)

        relay9 assertOrderBothDirection relay10
    }

    @Test
    fun test_compare_same_name() {
        val relay9a = Relay(mockedCity, "se9-wireguard", false)
        val relay9b = Relay(mockedCity, "se9-wireguard", false)

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun test_compare_only_numbers_in_name() {
        val relay001 = Relay(mockedCity, "001", false)
        val relay1 = Relay(mockedCity, "1", false)
        val relay3 = Relay(mockedCity, "3", false)
        val relay100 = Relay(mockedCity, "100", false)

        relay001 assertOrderBothDirection relay1
        relay001 assertOrderBothDirection relay3
        relay1 assertOrderBothDirection relay3
        relay3 assertOrderBothDirection relay100
    }

    @Test
    fun test_compare_without_numbers_in_name() {
        val relay9a = Relay(mockedCity, "se-wireguard", false)
        val relay9b = Relay(mockedCity, "se-wireguard", false)

        assertTrue(RelayNameComparator.compare(relay9a, relay9b) == 0)
        assertTrue(RelayNameComparator.compare(relay9b, relay9a) == 0)
    }

    @Test
    fun test_compare_with_trailing_zeros_in_name() {
        val relay001 = Relay(mockedCity, "se001-wireguard", false)
        val relay005 = Relay(mockedCity, "se005-wireguard", false)

        relay001 assertOrderBothDirection relay005
    }

    @Test
    fun test_compare_prefix_and_numbers() {
        val relayAr2 = Relay(mockedCity, "ar2-wireguard", false)
        val relayAr8 = Relay(mockedCity, "ar8-wireguard", false)
        val relaySe5 = Relay(mockedCity, "se5-wireguard", false)
        val relaySe10 = Relay(mockedCity, "se10-wireguard", false)

        relayAr2 assertOrderBothDirection relayAr8
        relayAr8 assertOrderBothDirection relaySe5
        relaySe5 assertOrderBothDirection relaySe10
    }

    @Test
    fun test_compare_suffix_and_numbers() {
        val relay2c = Relay(mockedCity, "se2-cloud", false)
        val relay2w = Relay(mockedCity, "se2-wireguard", false)

        relay2c assertOrderBothDirection relay2w
    }

    @Test
    fun test_compare_different_length() {
        val relay22a = Relay(mockedCity, "se22", false)
        val relay22b = Relay(mockedCity, "se22-wireguard", false)

        relay22a assertOrderBothDirection relay22b
    }

    private infix fun Relay.assertOrderBothDirection(other: Relay) {
        assertTrue(RelayNameComparator.compare(this, other) < 0)
        assertTrue(RelayNameComparator.compare(other, this) > 0)
    }
}
