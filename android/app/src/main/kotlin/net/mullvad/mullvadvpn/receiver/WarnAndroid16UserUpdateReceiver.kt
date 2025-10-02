package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import kotlin.getValue
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.util.goAsync
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import org.koin.java.KoinJavaComponent.inject

class WarnAndroid16UserUpdateReceiver : BroadcastReceiver(), KoinComponent {
    private val userPreferencesRepository by inject<UserPreferencesRepository>()

    override fun onReceive(context: Context?, intent: Intent?) {
        Logger.d("WarnAndroid16UserUpdateReceiver.onReceive")
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
