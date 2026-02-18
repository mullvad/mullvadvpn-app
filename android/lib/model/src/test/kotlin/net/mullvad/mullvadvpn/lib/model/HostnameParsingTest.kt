package net.mullvad.mullvadvpn.lib.model

import kotlin.test.assertEquals
import org.junit.jupiter.api.Test

class HostnameParsingTest {
    @Test
    fun `us-was-wg-002 should be correctly parsed`() {
        val parsedHostname = GeoLocationId.Hostname.from("us-was-wg-002")
        val country = GeoLocationId.Country("us")
        val city = GeoLocationId.City(country, "was")
        val hostname = GeoLocationId.Hostname(city, "wg-002")
        assertEquals("us", hostname.country.code)
        assertEquals("was", hostname.city.code)
        assertEquals("wg-002", hostname.code)
        assertEquals(hostname, parsedHostname)
    }
}
