package net.mullvad.mullvadvpn.compose.util

import android.content.Context
import androidx.compose.ui.text.input.KeyboardType

fun KeyboardType.Companion.numberPasswordInputType(context: Context): KeyboardType =
    if (isFireStick(context)) {
        Number
    } else {
        NumberPassword
    }

// see: https://developer.amazon.com/docs/fire-tv/identify-amazon-fire-tv-devices.html
private fun isFireStick(context: Context): Boolean =
    context.packageManager.hasSystemFeature("amazon.hardware.fire_tv")
