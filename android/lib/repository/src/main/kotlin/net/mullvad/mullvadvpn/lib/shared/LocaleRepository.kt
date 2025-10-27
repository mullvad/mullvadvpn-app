package net.mullvad.mullvadvpn.lib.repository

import android.content.res.Resources
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlin.also
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class LocaleRepository(val resources: Resources) {
    private val _currentLocale = MutableStateFlow(getLocale())
    val currentLocale: StateFlow<Locale?> = _currentLocale

    private fun getLocale(): Locale? = resources.configuration.locales.get(0)

    fun refreshLocale() {
        _currentLocale.value = getLocale().also { Logger.d("New locale: $it") }
    }
}
