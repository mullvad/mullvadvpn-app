package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.KeyStatusListener

class KeyStatusNotification(
    context: Context,
    authTokenCache: AuthTokenCache,
    private val keyStatusListener: KeyStatusListener
) : NotificationWithUrlWithToken(context, authTokenCache, R.string.wg_key_url) {
    private val failedToGenerateKey = context.getString(R.string.failed_to_generate_key)
    private val tooManyKeys = context.getString(R.string.too_many_keys)

    init {
        status = StatusLevel.Error
        title = context.getString(R.string.wireguard_error)
    }

    override fun onResume() {
        keyStatusListener.onKeyStatusChange.subscribe(this) { keyStatus ->
            jobTracker.newUiJob("updateKeyStatus") {
                updateKeyStatus(keyStatus)
            }
        }
    }

    override fun onPause() {
        keyStatusListener.onKeyStatusChange.unsubscribe(this)
    }

    private fun updateKeyStatus(keyStatus: KeygenEvent?) {
        when (keyStatus) {
            null -> shouldShow = false
            is KeygenEvent.NewKey -> shouldShow = false
            is KeygenEvent.TooManyKeys -> showTooManyKeys()
            is KeygenEvent.GenerationFailure -> showGenerationFailure()
        }

        update()
    }

    private fun showTooManyKeys() {
        onClick = openUrl
        message = tooManyKeys
        showIcon = true
        shouldShow = true
    }

    private fun showGenerationFailure() {
        onClick = null
        message = failedToGenerateKey
        showIcon = false
        shouldShow = true
    }
}
