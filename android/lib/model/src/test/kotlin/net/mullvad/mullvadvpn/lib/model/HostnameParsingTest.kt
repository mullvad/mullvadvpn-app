package net.mullvad.mullvadvpn.lib.model

import kotlin.test.assertEquals
import org.junit.jupiter.api.Test

class HostnameParsingTest {
    @Test
    fun `us-was-wg-002 should be correctly parsed`() {
        val input = "us-was-wg-002"
        val parsedHostname = GeoLocationId.Hostname.from(input)
        val country = GeoLocationId.Country("us")
        val city = GeoLocationId.City(country, "was")
        val hostname = GeoLocationId.Hostname(city, input)
        assertEquals("us", hostname.country.code)
        assertEquals("was", hostname.city.code)
        assertEquals(input, hostname.code)
        assertEquals(hostname, parsedHostname)
    }
}
