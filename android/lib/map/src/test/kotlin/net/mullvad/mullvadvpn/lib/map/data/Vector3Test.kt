package net.mullvad.mullvadvpn.lib.map.data

import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class Vector3Test {

    @ParameterizedTest
    @ValueSource(
        floats =
            [
                1.0000001f,
                1f,
                0.9999999f,
                0.9f,
                0.8f,
                0.7f,
                0.5f,
                0.0001f,
                0f,
                -0.5f,
                -0.9999999f,
                -1f,
                -1.0000001f,
            ]
    )
    fun `given a vector3 y should produce a valid latitude`(y: Float) {
        val vector = Vector3(0f, y, 0f)

        assertDoesNotThrow { vector.toLatLong() }
    }
}
