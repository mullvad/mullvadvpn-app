package net.mullvad.mullvadvpn.e2e.extension

import android.os.Bundle

fun Bundle.getRequiredArgument(argument: String): String {
    return getString(argument)
        ?: throw IllegalArgumentException("Missing required argument: $argument")
}
