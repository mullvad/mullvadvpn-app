package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue
import kotlin.test.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class LongitudeTest {

    @Test
    fun `create too high longitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Longitude(180.1f) }
    }

    @Test
    fun `create too low longitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Longitude(-180.1f) }
    }

    @Test
    fun `adding two positive latitude should result in the sum`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 30f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 + longFloat2, (longitude1 + longitude2).value)
    }

    @Test
    fun `adding two large positive latitude should result in the sum wrapped`() {
        val longFloat1 = 170f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 150f
        val longitude2 = Longitude(longFloat2)

        val expectedResult = -40f

        assertEquals(expectedResult, (longitude1 + longitude2).value)
    }

    @Test
    fun `adding two negative latitude should result in the sum wrapped`() {
        val longFloat1 = -80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = -40f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 + longFloat2, (longitude1 + longitude2).value)
    }

    @Test
    fun `subtracting two positive latitude should result in the sum`() {
        val longFloat1 = 80f
        val longitude1 = Longitude(longFloat1)
        val longFloat2 = 30f
        val longitude2 = Longitude(longFloat2)

        assertEquals(longFloat1 - longFloat2, (longitude1 - longitude2).value)
    }

    @Test
    fun `subtracting a large latitude should result in the sum wrapped`() {
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

        val absoluteLatitude2 = Longitude.fromFloat(longFloat2.absoluteValue)
        assertEquals(longitude1 + absoluteLatitude2, longitude1 - longitude2)
    }
}
