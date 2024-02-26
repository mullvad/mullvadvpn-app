package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue
import kotlin.test.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.api.assertThrows

class LongitudeTest {
    @Test
    fun `create longitude with longitude should work`() {
        assertDoesNotThrow { Longitude(80f) }
    }

    @Test
    fun `create longitude with negative longitude should work`() {
        assertDoesNotThrow { Longitude(-80f) }
    }

    @Test
    fun `create too high longitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Longitude(180.1f) }
    }

    @Test
    fun `create too low longitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Longitude(-180.1f) }
    }

    @Test
    fun `fromFloat should accept and wrap large value`() {
        val longFloat = 720f
        val longitude = Longitude.fromFloat(longFloat)

        assertEquals(0f, longitude.value)
    }

    @Test
    fun `fromFloat should accept and wrap large negative value`() {
        val longFloat = -720f
        val longitude = Longitude.fromFloat(longFloat)

        assertEquals(0f, longitude.value, 0f)
    }

    @Test
    fun `adding two positive longitude should result in the sum`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 30f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 + longFloat2, (longitude1 + longitude2).value)
    }

    @Test
    fun `adding two large positive longitude should result in the sum wrapped`() {
        val longFloat1 = 170f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 150f
        val longitude2 = Longitude(longFloat2)

        val expectedResult = -40f

        assertEquals(expectedResult, (longitude1 + longitude2).value)
    }

    @Test
    fun `adding two negative longitude should result in the sum wrapped`() {
        val longFloat1 = -80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = -40f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 + longFloat2, (longitude1 + longitude2).value)
    }

    @Test
    fun `subtracting two positive longitude should result in the sum`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 30f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 - longFloat2, (longitude1 - longitude2).value)
    }

    @Test
    fun `subtracting a large longitude should result in the sum wrapped`() {
        val longFloat1 = -30f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 170f
        val longitude2 = Longitude(longFloat2)

        val expectedResult = 160f

        assertEquals(expectedResult, (longitude1 - longitude2).value)
    }

    @Test
    fun `subtracting a negative latitude should result in same as addition`() {
        val longFloat1 = -80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = -40f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 + longFloat2.absoluteValue, (longitude1 - longitude2).value)
    }

    @Test
    fun `subtracting a large negative latitude should result in same as addition wrapped`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = -140f
        val longitude2 = Longitude(longFloat2)

        val absoluteLongitude2 = Longitude.fromFloat(longFloat2.absoluteValue)
        assertEquals(longitude1 + absoluteLongitude2, longitude1 - longitude2)
    }

    @Test
    fun `distanceTo with two positive longitudes should return distance`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 30f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 - longFloat2, longitude1.distanceTo(longitude2))
    }

    @Test
    fun `distanceTo with two negative longitudes should return distance`() {
        val longFloat1 = -80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = -30f
        val longitude2 = Longitude(longFloat2)

        val expectedValue = 50f

        assertEquals(expectedValue, longitude1.distanceTo(longitude2))
    }

    @Test
    fun `distanceTo with wrapping value should return shortest path as distance`() {
        val longFloat1 = -170f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 170f
        val longitude2 = Longitude(longFloat2)

        val expectedValue = 20f

        assertEquals(expectedValue, longitude1.distanceTo(longitude2))
    }
}
