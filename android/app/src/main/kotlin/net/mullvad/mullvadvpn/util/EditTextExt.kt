package net.mullvad.mullvadvpn.util

import android.view.KeyEvent
import android.view.inputmethod.EditorInfo
import android.widget.EditText

fun EditText.setOnEnterOrDoneAction(callback: () -> Unit) {
    setOnEditorActionListener { _, action, event ->
        if (action == EditorInfo.IME_ACTION_DONE || event?.keyCode == KeyEvent.KEYCODE_ENTER) {
            callback()
        }

        false
    }
}
