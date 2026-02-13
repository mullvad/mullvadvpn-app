package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import kotlin.getValue
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.receiver.util.goAsync
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class Android16UpdateWarningReceiver : BroadcastReceiver(), KoinComponent {
    private val userPreferencesRepository by inject<UserPreferencesRepository>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_MY_PACKAGE_REPLACED) {
            // Check that we run Android 16 (Baklava)
            goAsync {
                userPreferencesRepository.setShowAndroid16ConnectWarning(
                    Build.VERSION.SDK_INT == Build.VERSION_CODES.BAKLAVA
                )
            }
        }
    }
}
