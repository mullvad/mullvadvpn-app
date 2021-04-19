package net.mullvad.mullvadvpn.util

import android.util.Log
import android.view.View
import android.view.ViewGroup.MarginLayoutParams

fun View.setMargins(l: Int? = null, t: Int? = null, r: Int? = null, b: Int? = null) {
    if (this.layoutParams is MarginLayoutParams) {
        val p = this.layoutParams as MarginLayoutParams
        p.setMargins(l ?: p.leftMargin, t ?: p.topMargin, r ?: p.rightMargin, b ?: p.bottomMargin)
        this.requestLayout()
    } else {
        Log.w("mullvad", "setMargins is not supported")
    }
}
