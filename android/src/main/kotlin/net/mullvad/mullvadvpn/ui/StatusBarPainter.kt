package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.content.Context
import androidx.annotation.ColorInt

interface StatusBarPainter {
    fun getContext(): Context?
}

fun StatusBarPainter.paintStatusBar(@ColorInt color: Int) {
    (getContext() as Activity?)?.window?.statusBarColor = color
}
