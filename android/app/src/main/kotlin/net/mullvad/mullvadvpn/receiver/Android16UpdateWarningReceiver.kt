package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import kotlin.getValue
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.util.goAsync
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class Android16UpdateWarningReceiver : BroadcastReceiver(), KoinComponent {
    private val userPreferencesRepository by inject<UserPreferencesRepository>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_MY_PACKAGE_REPLACED) {
            // Check that we run Android 16 (Baklava)
            goAsync {
                userPreferencesRepository.setShowAndroid16ConnectWarning(
                    android.os.Build.VERSION.SDK_INT == android.os.Build.VERSION_CODES.BAKLAVA
                )
            }
        }
    }
}
