package net.mullvad.mullvadvpn.ui.extension

import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.ui.MainActivity

fun Fragment.requireMainActivity(): MainActivity {
    return if (this.activity is MainActivity) {
        this.activity as MainActivity
    } else {
        throw IllegalStateException(
            "Fragment $this not attached to ${MainActivity::class.simpleName}."
        )
    }
}
