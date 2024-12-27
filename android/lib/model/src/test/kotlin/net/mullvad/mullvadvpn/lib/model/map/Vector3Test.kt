package net.mullvad.mullvadvpn.lib.model.map

import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude
import org.junit.jupiter.api.Test

class Vector3Test {
    @Test
    fun `Y-axis center test`() {
        assertVector3Equals(Y_POSITIVE_CENTER.toVector3(), Vector3(0f, 1f, 0f))
        assertVector3Equals(Y_NEGATIVE_CENTER.toVector3(), Vector3(0f, -1f, 0f))
    }

    @Test
    fun `Z-axis center test`() {
        assertVector3Equals(Z_POSITIVE_CENTER.toVector3(), Vector3(0f, 0f, 1f))
        assertVector3Equals(Z_NEGATIVE_CENTER.toVector3(), Vector3(0f, 0f, -1f))
    }

    @Test
    fun `X-axis center test`() {
        assertVector3Equals(X_POSITIVE_CENTER.toVector3(), Vector3(-1f, 0f, 0f))
        assertVector3Equals(X_NEGATIVE_CENTER.toVector3(), Vector3(1f, 0f, 0f))
    }

    @Test
    fun `Y-axis center to LatLng test`() {
        assertLatLngEquals(Vector3(0f, 1f, 0f).toLatLng(), Y_POSITIVE_CENTER)
        assertLatLngEquals(Vector3(0f, -1f, 0f).toLatLng(), Y_NEGATIVE_CENTER)
    }

    @Test
    fun `Z-axis center to LatLng test`() {
        assertLatLngEquals(Vector3(0f, 0f, 1f).toLatLng(), Z_POSITIVE_CENTER)
        assertLatLngEquals(Vector3(0f, 0f, -1f).toLatLng(), Z_NEGATIVE_CENTER)
    }

    @Test
    fun `X-axis center to LatLng test`() {
        assertLatLngEquals(Vector3(-1f, 0f, 0f).toLatLng(), X_POSITIVE_CENTER)
        assertLatLngEquals(Vector3(1f, 0f, 0f).toLatLng(), X_NEGATIVE_CENTER)
    }

    companion object {
        // NORTH POLE
        val Y_POSITIVE_CENTER = LatLong(Latitude(90f), Longitude(0f))
        // SOUTH POLE
        val Y_NEGATIVE_CENTER = LatLong(Latitude(-90f), Longitude(0f))

        val Z_POSITIVE_CENTER = LatLong(Latitude(0f), Longitude(0f))
        val Z_NEGATIVE_CENTER = LatLong(Latitude(0f), Longitude.fromFloat(180f))

        val X_NEGATIVE_CENTER = LatLong(Latitude(0f), Longitude.fromFloat(-90f))
        val X_POSITIVE_CENTER = LatLong(Latitude(0f), Longitude.fromFloat(90f))
    }

    fun assertVector3Equals(expected: Vector3, actual: Vector3) {
        assertEquals(expected.x, actual.x, 0.0001f)
        assertEquals(expected.y, actual.y, 0.0001f)
        assertEquals(expected.z, actual.z, 0.0001f)
    }

    fun assertLatLngEquals(expected: LatLong, actual: LatLong) {
        assertEquals(expected.latitude.distanceTo(actual.latitude), 0f, 0.0001f)
        assertEquals(expected.longitude.distanceTo(actual.longitude), 0f, 0.0001f)
    }
}
