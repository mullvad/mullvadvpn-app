package net.mullvad.mullvadvpn.model

import kotlin.math.sqrt
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class LatLongTest {

    @Test
    fun `degree distance between two LatLong should be same as hypotenuse`() {
        val latLong1 = LatLong(Latitude(30f), Longitude(40f))
        val latLong2 = LatLong(Latitude(-40f), Longitude(170f))

        val latDiff = latLong1.latitude.distanceTo(latLong2.latitude)
        val longDiff = latLong1.longitude.distanceTo(latLong2.longitude)
        val hypotenuse = sqrt(latDiff * latDiff + longDiff * longDiff)

        assertEquals(hypotenuse, latLong1.degreeDistanceTo(latLong2))
    }

    @Test
    fun `ensure seppDistance respects lateral value`() {
        // Malmö & New York is a shorter distance than Malmö & Johannesburg, but the degree
        // difference is larger since they are at a higher latitude.

        // Malmo 55.6050° N, 13.0038° E
        val malmo = LatLong(Latitude(55.6050f), Longitude(13.0038f))

        // New York 40.7128° N, 74.0060° W
        val newYork = LatLong(Latitude(40.7128f), Longitude(-74.0060f))

        // Johannesburg 26.2041° S, 28.0473° E
        val johannesburg = LatLong(Latitude(-26.2041f), Longitude(28.0473f))

        val malmoToNewYork = malmo.seppDistanceTo(newYork)
        val malmoToJohannesburg = malmo.seppDistanceTo(johannesburg)

        assertTrue { malmoToNewYork < malmoToJohannesburg }
    }
}
