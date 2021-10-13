package net.mullvad.mullvadvpn.ui

import android.app.Activity
import androidx.annotation.ColorInt

interface StatusBarPainter : SystemPainter

fun StatusBarPainter.paintStatusBar(@ColorInt color: Int) {
    (getContext() as Activity?)?.window?.statusBarColor = color
}
