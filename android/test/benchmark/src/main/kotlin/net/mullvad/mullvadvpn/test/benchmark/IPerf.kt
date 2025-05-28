package net.mullvad.mullvadvpn.test.benchmark

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import java.io.File
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromStream
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetIp
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetPassword
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetPort
import net.mullvad.mullvadvpn.test.benchmark.constant.getTargetUsername
import net.mullvad.mullvadvpn.test.benchmark.model.IperfResult

@OptIn(ExperimentalSerializationApi::class)
fun runIperf3(context: Context, targetContext: Context): IperfResult {
    val publicKeyPath = getPublicKeyFilePath(context, targetContext)
    val iperf = File(context.applicationInfo.nativeLibraryDir + "/iperf3.21")
    iperf.setExecutable(true)
    val process =
        ProcessBuilder(
                iperf.absolutePath,
                "-c",
                InstrumentationRegistry.getArguments().getTargetIp(),
                "-p",
                InstrumentationRegistry.getArguments().getTargetPort(),
                "--json",
            )
            .also {
                if (InstrumentationRegistry.getArguments().getTargetUsername().isNotEmpty()) {
                    it.command()
                        .addAll(
                            listOf(
                                "--username",
                                InstrumentationRegistry.getArguments().getTargetUsername(),
                                "--rsa-public-key-path",
                                publicKeyPath,
                            )
                        )
                    val env = it.environment()
                    env[PASSWORD_ENV] = InstrumentationRegistry.getArguments().getTargetPassword()
                }
            }
            .redirectErrorStream(true)
            .start()

    val exitValue = process.waitFor()

    if (exitValue != 0) {
        val output = process.inputStream.bufferedReader().readText()
        Logger.e { "iperf3 exited with non-zero exit code $exitValue. Output: $output" }
        error("iperf3 exited with non-zero exit code $exitValue")
    }

    return Json.decodeFromStream<IperfResult>(process.inputStream)
}

fun IperfResult.summarize(): String = buildString {
    appendLine("# Speed #")
    if (end.sumReceived != null)
        appendLine("Receive: ${end.sumReceived.bitsPerSecond / 1000000} Mbit/s")
    if (end.sumSent != null) appendLine("Send: ${end.sumSent.bitsPerSecond / 1000000} Mbit/s")
    appendLine("# CPU #")
    if (end.cpuUtilizationPercent != null) appendLine("User: ${end.cpuUtilizationPercent.hostUser}")
    if (end.cpuUtilizationPercent != null)
        appendLine("System: ${end.cpuUtilizationPercent.hostSystem}")
    if (end.cpuUtilizationPercent != null)
        appendLine("Total: ${end.cpuUtilizationPercent.hostTotal}")
}

private fun getPublicKeyFilePath(context: Context, targetContext: Context): String {
    val outFile = File(targetContext.filesDir, "iperf-server-public.pem")

    outFile.parentFile?.mkdirs()

    if (!outFile.exists()) {
        context.assets.open("iperf3/iperf3.pem").use { input ->
            outFile.outputStream().use { output ->
                input.copyTo(output)
            }
        }
    }

    return outFile.absolutePath
}

private const val PASSWORD_ENV = "IPERF3_PASSWORD"
