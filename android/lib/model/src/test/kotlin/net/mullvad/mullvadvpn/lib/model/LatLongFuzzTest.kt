package net.mullvad.mullvadvpn.lib.model

import com.code_intelligence.jazzer.api.FuzzedDataProvider
import com.code_intelligence.jazzer.junit.FuzzTest

class LatLongFuzzTest {
    @FuzzTest
    fun fromFloat(data: FuzzedDataProvider) {
        Latitude.fromFloat(data.consumeFloat())
        Longitude.fromFloat(data.consumeFloat())
    }
}
