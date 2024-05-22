package net.mullvad.mullvadvpn.compose.util

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.VpnService
import androidx.activity.result.contract.ActivityResultContract

class RequestVpnPermission : ActivityResultContract<Unit, Boolean>() {
    override fun createIntent(context: Context, input: Unit): Intent {
        // We expect this permission to only be requested when the permission is missing, however,
        // if it for some reason is called incorrectly we should return an empty intent so we avoid
        // a crash.
        return VpnService.prepare(context) ?: Intent()
    }

    override fun parseResult(resultCode: Int, intent: Intent?): Boolean {
        return resultCode == Activity.RESULT_OK
    }
}
