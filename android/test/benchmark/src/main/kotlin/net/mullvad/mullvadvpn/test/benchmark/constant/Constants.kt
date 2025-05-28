package net.mullvad.mullvadvpn.test.benchmark.constant

import android.os.Bundle
import net.mullvad.mullvadvpn.test.benchmark.BuildConfig

const val LOG_TAG = "mullvad-benchmark"

fun Bundle.getRequiredArgument(argument: String): String {
    return getString(argument)
        ?: throw IllegalArgumentException("Missing required argument: $argument")
}

fun Bundle.getPartnerAuth() = getString("mullvad.test.e2e.${BuildConfig.FLAVOR}.partnerAuth")

fun Bundle.getValidAccountNumber() =
    getRequiredArgument("mullvad.test.e2e.${BuildConfig.FLAVOR}.accountNumber.valid")

fun Bundle.getTargetIp() = getRequiredArgument("mullvad.test.benchmark.target.ip")

fun Bundle.getTargetPort() = getRequiredArgument("mullvad.test.benchmark.target.port")

fun Bundle.getInvalidAccountNumber() =
    getRequiredArgument("mullvad.test.e2e.${BuildConfig.FLAVOR}.accountNumber.invalid")

fun Bundle.isRaasEnabled(): Boolean =
    getRequiredArgument("mullvad.test.e2e.config.raas.enable").toBoolean()

fun Bundle.isHighlyRateLimitedTestsEnabled(): Boolean =
    getRequiredArgument("mullvad.test.e2e.config.runHighlyRateLimitedTests").toBoolean()

fun Bundle.getRaasHost() = getRequiredArgument("mullvad.test.e2e.config.raas.host")

fun Bundle.getTrafficGeneratorHost(): String =
    getRequiredArgument("mullvad.test.e2e.config.raas.trafficGenerator.target.host")

fun Bundle.getTrafficGeneratorPort(): Int =
    getRequiredArgument("mullvad.test.e2e.config.raas.trafficGenerator.target.port").toInt()
