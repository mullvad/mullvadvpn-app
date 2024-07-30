package net.mullvad.mullvadvpn.compose.util

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.VpnService
import androidx.activity.result.contract.ActivityResultContract

class RequestVpnPermission : ActivityResultContract<Unit, Boolean>() {
    override fun createIntent(context: Context, input: Unit): Intent {
        return VpnService.prepare(context)!!
    }

    override fun parseResult(resultCode: Int, intent: Intent?): Boolean {
        return resultCode == Activity.RESULT_OK
    }

    // We expect this permission to only be requested when the permission is missing. However,
    // if it for some reason is called incorrectly we will skip the call to create intent
    // to avoid crashing. The app will then proceed as the user accepted the permission.
    override fun getSynchronousResult(context: Context, input: Unit): SynchronousResult<Boolean>? {
        return if (VpnService.prepare(context) == null) {
            SynchronousResult(true)
        } else {
            null
        }
    }
}
