package net.mullvad.mullvadvpn.repository

import android.content.res.Resources
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow

class LocaleRepository(val resources: Resources) {
    val currentLocale = MutableStateFlow(getLocale())

    private fun getLocale(): Locale? = resources.configuration.locales.get(0)

    fun refreshLocale() {
        currentLocale.value = getLocale()
    }
}
