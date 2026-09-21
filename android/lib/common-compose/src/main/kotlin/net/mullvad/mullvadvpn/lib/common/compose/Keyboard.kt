package net.mullvad.mullvadvpn.lib.common.compose

import android.content.Context
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.key
import androidx.compose.ui.text.input.KeyboardType

fun KeyboardType.Companion.accountNumberKeyboardType(context: Context): KeyboardType =
    if (isFireStick(context)) {
        Number
    } else {
        NumberPassword
    }

// See: https://developer.amazon.com/docs/fire-tv/identify-amazon-fire-tv-devices.html
private fun isFireStick(context: Context): Boolean =
    context.packageManager.hasSystemFeature("amazon.hardware.fire_tv")

val KeyEvent.isDirectionRight: Boolean
    get() =
        key == Key.DirectionRight ||
            nativeKeyEvent.keyCode == android.view.KeyEvent.KEYCODE_DPAD_RIGHT

val KeyEvent.isDirectionCenter: Boolean
    get() =
        key == Key.DirectionCenter ||
            nativeKeyEvent.keyCode == android.view.KeyEvent.KEYCODE_DPAD_CENTER
