package net.mullvad.mullvadvpn.repository

import android.content.res.Resources
import android.os.Build
import android.util.Log
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow

class LocaleRepository(val resources: Resources) {
    val currentLocale = MutableStateFlow(getLocale())

    private fun getLocale(): Locale? = resources.configuration.locales.get(0)

    fun refreshLocale() {
        Logger.d("AppLang: Refreshing locale")
        currentLocale.value = getLocale().also {
            Logger.d("AppLang: New locale: $it")
        }
    }
}
