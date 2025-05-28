package net.mullvad.mullvadvpn.test.benchmark

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import java.io.File
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromStream
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetIp
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetPort
import net.mullvad.mullvadvpn.test.benchmark.model.IperfResult

@OptIn(ExperimentalSerializationApi::class)
fun runIperf3(context: Context): IperfResult {
    val iperf = File(context.applicationInfo.nativeLibraryDir + "/iperf3.18")
    iperf.setExecutable(true)
    val process =
        ProcessBuilder(iperf.absolutePath, "-c", InstrumentationRegistry.getArguments().getTargetIp(), "-p",
            InstrumentationRegistry.getArguments().getTargetPort(), "--json").start()
    return Json.decodeFromStream<IperfResult>(process.inputStream)
}

fun IperfResult.summarize(): String = buildString {
    appendLine("# Speed #")
    appendLine("Receive: ${end.sumReceived.bitsPerSecond / 1000000} Mbit/s")
    appendLine("Send: ${end.sumSent.bitsPerSecond / 1000000} Mbit/s")
    appendLine("# CPU #")
    appendLine("User: ${end.cpuUtilizationPercent.hostUser}")
    appendLine("System: ${end.cpuUtilizationPercent.hostSystem}")
    appendLine("Total: ${end.cpuUtilizationPercent.hostTotal}")
}
