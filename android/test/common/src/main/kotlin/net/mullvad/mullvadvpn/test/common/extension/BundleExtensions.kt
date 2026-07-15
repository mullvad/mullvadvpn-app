package net.mullvad.mullvadvpn.test.common.extension

import android.os.Bundle

fun Bundle.getRequiredArgument(argument: String): String {
    return getString(argument)
        ?: throw IllegalArgumentException("Missing required argument: $argument")
}
