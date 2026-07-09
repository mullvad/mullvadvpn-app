package net.mullvad.mullvadvpn.test.benchmark.constant

import android.os.Bundle
import net.mullvad.mullvadvpn.test.benchmark.BuildConfig
import net.mullvad.mullvadvpn.test.common.extension.getRequiredArgument

const val LOG_TAG = "mullvad-benchmark"

fun Bundle.getPartnerAuth() = getString("mullvad.test.e2e.${BuildConfig.FLAVOR}.partnerAuth")

fun Bundle.getValidAccountNumber() =
    getRequiredArgument("mullvad.test.e2e.${BuildConfig.FLAVOR}.accountNumber.valid")

fun Bundle.getTargetIp(): String = getString("mullvad.test.benchmark.target.ip", "10.64.0.1")

fun Bundle.getTargetPort(): String = getString("mullvad.test.benchmark.target.port", "5201")

fun Bundle.getTargetUsername() = getString("mullvad.test.benchmark.target.username") ?: ""

fun Bundle.getTargetPassword() = getString("mullvad.test.benchmark.target.password") ?: ""
