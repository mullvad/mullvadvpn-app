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

        assertTrue(RelayNameComparator.compare(relay9, relay10) < 0)
        assertTrue(RelayNameComparator.compare(relay10, relay9) > 0)
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

        assertTrue(RelayNameComparator.compare(relay001, relay1) == 0)
        assertTrue(RelayNameComparator.compare(relay1, relay001) == 0)
        assertTrue(RelayNameComparator.compare(relay001, relay3) < 0)
        assertTrue(RelayNameComparator.compare(relay1, relay3) < 0)
        assertTrue(RelayNameComparator.compare(relay3, relay100) < 0)
        assertTrue(RelayNameComparator.compare(relay3, relay001) > 0)
        assertTrue(RelayNameComparator.compare(relay3, relay1) > 0)
        assertTrue(RelayNameComparator.compare(relay100, relay3) > 0)
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

        assertTrue(RelayNameComparator.compare(relay001, relay005) < 0)
        assertTrue(RelayNameComparator.compare(relay005, relay001) > 0)
    }

    @Test
    fun test_compare_prefix_and_numbers() {
        val relay2 = Relay(mockedCity, "ar2-wireguard", false)
        val relay8 = Relay(mockedCity, "ar8-wireguard", false)
        val relay5 = Relay(mockedCity, "se5-wireguard", false)
        val relay10 = Relay(mockedCity, "se10-wireguard", false)

        assertTrue(RelayNameComparator.compare(relay2, relay8) < 0)
        assertTrue(RelayNameComparator.compare(relay8, relay5) < 0)
        assertTrue(RelayNameComparator.compare(relay5, relay10) < 0)
        assertTrue(RelayNameComparator.compare(relay8, relay2) > 0)
        assertTrue(RelayNameComparator.compare(relay5, relay8) > 0)
        assertTrue(RelayNameComparator.compare(relay10, relay5) > 0)
    }

    @Test
    fun test_compare_suffix_and_numbers() {
        val relay2c = Relay(mockedCity, "se2-cloud", false)
        val relay2w = Relay(mockedCity, "se2-wireguard", false)

        assertTrue(RelayNameComparator.compare(relay2c, relay2w) < 0)
        assertTrue(RelayNameComparator.compare(relay2w, relay2c) > 0)
    }

    @Test
    fun test_compare_different_length() {
        val relay22a = Relay(mockedCity, "se22", false)
        val relay22b = Relay(mockedCity, "se22-wireguard", false)

        assertTrue(RelayNameComparator.compare(relay22a, relay22b) < 0)
        assertTrue(RelayNameComparator.compare(relay22b, relay22a) > 0)
    }
}
