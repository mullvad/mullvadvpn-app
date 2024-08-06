package net.mullvad.mullvadvpn.lib.shared

import android.content.res.Resources
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlin.also
import kotlinx.coroutines.flow.MutableStateFlow

class LocaleRepository(val resources: Resources) {
    val currentLocale = MutableStateFlow(getLocale())

    private fun getLocale(): Locale? = resources.configuration.locales.get(0)

    fun refreshLocale() {
        Logger.d("AppLang: Refreshing locale")
        currentLocale.value = getLocale().also { Logger.d("AppLang: New locale: $it") }
    }
}
