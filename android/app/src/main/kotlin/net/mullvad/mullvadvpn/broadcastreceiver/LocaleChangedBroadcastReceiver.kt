package net.mullvad.mullvadvpn.broadcastreceiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import net.mullvad.mullvadvpn.repository.LocaleRepository
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class LocaleChangedBroadcastReceiver: BroadcastReceiver(), KoinComponent {
    private val localeRepository by inject<LocaleRepository>()

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_LOCALE_CHANGED) {
            localeRepository.refreshLocale()
        }
    }
}
