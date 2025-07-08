package net.mullvad.mullvadvpn.test.e2e.constant

import android.os.Bundle
import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument

const val LOG_TAG = "mullvad-e2e"

fun Bundle.getPartnerAuth() =
    InstrumentationRegistry.getArguments()
        .getString("mullvad.test.e2e.${BuildConfig.FLAVOR_infrastructure}.partnerAuth")

fun Bundle.getValidAccountNumber() =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument(
            "mullvad.test.e2e.${BuildConfig.FLAVOR_infrastructure}.accountNumber.valid"
        )

fun Bundle.getInvalidAccountNumber() =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument(
            "mullvad.test.e2e.${BuildConfig.FLAVOR_infrastructure}.accountNumber.invalid"
        )

fun Bundle.isBillingEnabled(): Boolean =
    InstrumentationRegistry.getArguments()
        .getString("mullvad.test.e2e.config.billing.enable", "false")
        .toBoolean()

fun Bundle.isRaasEnabled(): Boolean =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.config.raas.enable")
        .toBoolean()

fun Bundle.isHighlyRateLimitedTestsEnabled(): Boolean =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.config.runHighlyRateLimitedTests")
        .toBoolean()

fun Bundle.getRaasHost() =
    InstrumentationRegistry.getArguments().getRequiredArgument("mullvad.test.e2e.config.raas.host")

fun Bundle.getTrafficGeneratorHost(): String =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.config.raas.trafficGenerator.target.host")

fun Bundle.getTrafficGeneratorPort(): Int =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.config.raas.trafficGenerator.target.port")
        .toInt()
