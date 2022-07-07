package net.mullvad.mullvadvpn.ui.extension

import android.content.Context
import android.content.Intent
import android.net.Uri
import net.mullvad.mullvadvpn.R

fun Context.openAccountPageInBrowser(authToken: String) {
    startActivity(
        Intent(
            Intent.ACTION_VIEW,
            Uri.parse(getString(R.string.account_url) + "?token=$authToken")
        )
    )
}
