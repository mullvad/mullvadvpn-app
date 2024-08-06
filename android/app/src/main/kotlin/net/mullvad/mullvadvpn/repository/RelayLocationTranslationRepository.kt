package net.mullvad.mullvadvpn.repository

import android.content.res.Resources
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow

class RelayLocationTranslationRepository(val localeRepository: LocaleRepository, val resources: Resources) {
    val currentTranslations = MutableStateFlow<Map<String, String>>(mapOf())

    init {
        localeRepository.currentLocale.collect { refreshTranslations() }
    }

    fun refreshTranslations() {
        

    }

}
