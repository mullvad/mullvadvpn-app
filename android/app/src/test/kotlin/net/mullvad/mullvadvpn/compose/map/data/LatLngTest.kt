package net.mullvad.mullvadvpn.compose.map.data

import org.junit.jupiter.api.Test

class LatLngTest {

    @Test
    fun distanceTo() {
        val gothenburgLatLng = LatLng(Latitude(57.7065f), Longitude(11.967f))
        val stockholmLatLng = LatLng(Latitude(59.3293f), Longitude(18.0686f))
        val distance = gothenburgLatLng.distanceTo(stockholmLatLng)
        assert(distance > 2.0)
    }

    @Test
    fun distanceToWrapping() {
        val point1 = LatLng(Latitude(0f), Longitude(-179f))
        val point2 = LatLng(Latitude(0f), Longitude(179f))
        val distance = point1.distanceTo(point2)

        assert(distance == 2.0)
    }

    @Test
    fun distanceToWrappingOpposite() {
        val point1 = LatLng(Latitude(0f), Longitude(179f))
        val point2 = LatLng(Latitude(0f), Longitude(-179f))
        val distance = point1.distanceTo(point2)
        assert(distance == 2.0)
    }
}
