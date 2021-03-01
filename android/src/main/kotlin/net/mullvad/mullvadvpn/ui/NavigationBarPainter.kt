package net.mullvad.mullvadvpn.ui

import android.app.Activity
import androidx.annotation.ColorInt

interface NavigationBarPainter : SystemPainter

fun NavigationBarPainter.paintNavigationBar(@ColorInt color: Int) {
    (getContext() as Activity?)?.window?.navigationBarColor = color
}
