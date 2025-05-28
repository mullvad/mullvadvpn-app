package net.mullvad.mullvadvpn.test.benchmark

import co.touchlab.kermit.Logger
import java.io.BufferedReader
import java.io.File
import java.io.IOException
import java.io.InputStreamReader
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.test.benchmark.model.IperfResult
import org.junit.jupiter.api.Test

class IperfResultTest : BenchmarkTest() {

    @Test
    fun testIPerf() {
        val result = runIperf3(context)
        Logger.d("!!! TESTING !!!")
        Logger.d(result.summarize())
    }
}
