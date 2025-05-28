package net.mullvad.mullvadvpn.test.benchmark

import co.touchlab.kermit.Logger
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class IperfResultTest : BenchmarkTest() {

    @Test
    @Disabled("Used for developing iPerf3 tests")
    fun testIPerf() {
        val result = runIperf3(context, targetContext)
        Logger.d("!!! TESTING !!!")
        if (result.error != null) {
            Logger.e("Error: ${result.error}")
        }
        Logger.d(result.summarize())
    }
}
