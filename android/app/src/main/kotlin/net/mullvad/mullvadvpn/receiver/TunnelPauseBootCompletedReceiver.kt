package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.repository.TunnelPauseRepository
import net.mullvad.mullvadvpn.receiver.util.goAsync
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class TunnelPauseBootCompletedReceiver : BroadcastReceiver(), KoinComponent {
    private val tunnelPauseRepository by inject<TunnelPauseRepository>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_BOOT_COMPLETED) {
            Logger.d("Rescheduling pause alarm from TunnelPauseBootCompletedReceiver")
            goAsync { tunnelPauseRepository.rescheduleAlarm() }
        }
    }
}
