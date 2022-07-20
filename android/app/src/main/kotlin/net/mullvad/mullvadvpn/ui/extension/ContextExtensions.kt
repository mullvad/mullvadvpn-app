package net.mullvad.mullvadvpn.ui.extension

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.MainActivity

fun Context.openAccountPageInBrowser(authToken: String) {
    startActivity(
        Intent(
            Intent.ACTION_VIEW,
            Uri.parse(getString(R.string.account_url) + "?token=$authToken")
        )
    )
}

fun Fragment.requireMainActivity(): MainActivity {
    return if (this.activity is MainActivity) {
        this.activity as MainActivity
    } else {
        throw IllegalStateException(
            "Fragment $this not attached to ${MainActivity::class.simpleName}."
        )
    }
}
