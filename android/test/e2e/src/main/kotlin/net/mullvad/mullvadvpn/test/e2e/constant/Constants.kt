package net.mullvad.mullvadvpn.test.e2e.constant

import android.os.Bundle
import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument

const val LOG_TAG = "mullvad-e2e"
const val PARTNER_AUTH = "partner_auth"

fun Bundle.getValidAccountNumber() =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("test.e2e.${BuildConfig.FLAVOR_infrastructure}.accountNumber.valid")

fun Bundle.getInvalidAccountNumber() =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("test.e2e.${BuildConfig.FLAVOR_infrastructure}.accountNumber.invalid")

fun Bundle.isRaasEnabled(): Boolean =
    InstrumentationRegistry.getArguments().getBoolean("test.e2e.config.raas.enable", false)

fun Bundle.isHighlyRateLimitedTestsEnabled(): Boolean =
    InstrumentationRegistry.getArguments()
        .getBoolean("test.e2e.config.runHighlyRateLimitedTests", false)

fun Bundle.getRaasHost() =
    InstrumentationRegistry.getArguments().getRequiredArgument("test.e2e.config.raas.host")

fun Bundle.getTrafficGeneratorHost(): String =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("test.e2e.config.raas.trafficGenerator.target.host")

fun Bundle.getTrafficGeneratorPort(): Int =
    InstrumentationRegistry.getArguments()
        .getInt("test.e2e.config.raas.trafficGenerator.target.port", 80)

val DOMAIN =
    when (BuildConfig.FLAVOR_infrastructure) {
        "stagemole" -> "stagemole.eu"
        else -> "mullvad.net"
    }
