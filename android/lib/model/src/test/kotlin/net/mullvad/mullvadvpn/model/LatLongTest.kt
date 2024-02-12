package net.mullvad.mullvadvpn.model

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import kotlin.math.sqrt

class LatLongTest {

    @Test
    fun `distance between two LatLong should be same as hypotenuse`() {
        val latLong1 = LatLong(Latitude(30f), Longitude(40f))
        val latLong2 = LatLong(Latitude(-40f), Longitude(170f))

        val latDiff = latLong1.latitude.distanceTo(latLong2.latitude)
        val longDiff = latLong1.longitude.distanceTo(latLong2.longitude)
        val hypotenuse = sqrt(latDiff * latDiff + longDiff * longDiff)

        assertEquals(hypotenuse, latLong1.distanceTo(latLong2))
    }
}
