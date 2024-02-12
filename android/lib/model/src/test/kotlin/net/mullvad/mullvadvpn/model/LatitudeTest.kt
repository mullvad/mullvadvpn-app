package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue
import kotlin.test.assertEquals
import org.junit.jupiter.api.Assertions.assertDoesNotThrow
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class LatitudeTest {
    @Test
    fun `creating a valid latitude should work`() {
        assertDoesNotThrow { Latitude(30f) }
    }

    @Test
    fun `creating a valid negative latitude should work`() {
        assertDoesNotThrow { Latitude(-30f) }
    }

    @Test
    fun `create with too high latitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Latitude(90.1f) }
    }

    @Test
    fun `create with too low latitude should give IllegalArgumentException`() {
        assertThrows<IllegalArgumentException> { Latitude(-90.1f) }
    }

    @Test
    fun `fromFloat should accept and wrap large value`() {
        val longFloat = 400f
        val longitude = Latitude.fromFloat(longFloat)

        assertEquals(40f, longitude.value)
    }

    @Test
    fun `fromFloat should accept and support half-wrap`() {
        val longFloat = 100f
        val longitude = Latitude.fromFloat(longFloat)

        assertEquals(80f, longitude.value)
    }

    @Test
    fun `fromFloat should accept and support negative half-wrap`() {
        val longFloat = -100f
        val longitude = Latitude.fromFloat(longFloat)

        assertEquals(-80f, longitude.value)
    }

    @Test
    fun `adding two positive latitude should result in the sum`() {
        val latFloat1 = 20f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = 30f
        val latitude2 = Latitude(latFloat2)

        assertEquals(latFloat1 + latFloat2, (latitude1 + latitude2).value)
    }

    @Test
    fun `adding two large positive latitude should result in the sum wrapped`() {
        val latFloat1 = 70f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = 50f
        val latitude2 = Latitude(latFloat2)

        val expectedResult = 60f

        assertEquals(expectedResult, (latitude1 + latitude2).value)
    }

    @Test
    fun `adding two negative latitude should result in the sum`() {
        val latFloat1 = -20f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = -40f
        val latitude2 = Latitude(latFloat2)

        assertEquals(latFloat1 + latFloat2, (latitude1 + latitude2).value)
    }

    @Test
    fun `adding two large negative latitude should result in the sum`() {
        val latFloat1 = -70f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = -50f
        val latitude2 = Latitude(latFloat2)

        val expectedResult = -60f

        assertEquals(expectedResult, (latitude1 + latitude2).value)
    }

    @Test
    fun `subtracting two positive latitude should result in the sum`() {
        val latFloat1 = 80f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = 30f
        val latitude2 = Latitude(latFloat2)

        assertEquals(latFloat1 - latFloat2, (latitude1 - latitude2).value)
    }

    @Test
    fun `subtracting a large latitude should result in the sum wrapped`() {
        val latFloat1 = -30f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = 80f
        val latitude2 = Latitude(latFloat2)

        val expectedResult = -70f

        assertEquals(expectedResult, (latitude1 - latitude2).value)
    }

    @Test
    fun `subtracting a negative latitude should result in same as addition`() {
        val latFloat1 = -30f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = -40f
        val latitude2 = Latitude(latFloat2)

        assertEquals(latFloat1 + latFloat2.absoluteValue, (latitude1 - latitude2).value)
    }

    @Test
    fun `subtracting a large negative latitude should result in same as addition wrapped`() {
        val latFloat1 = 80f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = -90f
        val latitude2 = Latitude(latFloat2)

        val absoluteLatitude2 = Latitude.fromFloat(latFloat2.absoluteValue)

        assertEquals(latitude1 + absoluteLatitude2, latitude1 - latitude2)
    }

    @Test
    fun `distanceTo with two positive latitudes`() {
        val latFloat1 = 80f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = 30f
        val latitude2 = Latitude(latFloat2)

        assertEquals(latFloat1 - latFloat2, latitude1.distanceTo(latitude2))
    }

    @Test
    fun `distanceTo with two negative latitudes`() {
        val latFloat1 = -80f
        val latitude1 = Latitude(latFloat1)
        val latFloat2 = -30f
        val latitude2 = Latitude(latFloat2)

        val expectedValue = 50f

        assertEquals(expectedValue, latitude1.distanceTo(latitude2))
    }
}
