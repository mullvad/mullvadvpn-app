package net.mullvad.mullvadvpn.repository

import android.content.Context
import android.content.res.Configuration
import android.content.res.XmlResourceParser
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R

class RelayLocationTranslationRepository(
    val context: Context,
    val localeRepository: LocaleRepository,
    scope: CoroutineScope
) {
    val translationTable = MutableStateFlow<Map<String, String>>(mapOf())

    private val defaultTranslation: Map<String, String>

    init {
        val defaultConfiguration = defaultConfiguration()
        val confContext = context.createConfigurationContext(defaultConfiguration)
        val defaultTranslationXml = confContext.resources.getXml(R.xml.relay_locations)
        defaultTranslation = loadRelayTranslation(defaultTranslationXml)
        Logger.d("AppLang: Default translation = $defaultTranslation")

        scope.launch { localeRepository.currentLocale.collect { dodo(it) } }

    }

    private fun dodo(locale: Locale?) {
        Logger.d("AppLang: Updating based on current locale to $locale")
        if (locale == null || locale.language == DEFAULT_LANGUAGE)
            translationTable.value = emptyMap()
        else {
            // Load current translations
            val xml = context.resources.getXml(R.xml.relay_locations)
            val translation = loadRelayTranslation(xml)


            translationTable.value =
                translation.entries.associate { (id, name) -> defaultTranslation[id]!! to name }.also {
                    Logger.d("AppLang: New translationTable: $it")
                }
        }
    }

    private fun loadRelayTranslation(xml: XmlResourceParser): Map<String, String> {
        val translation = mutableMapOf<String, String>()
        while (xml.eventType != XmlResourceParser.END_DOCUMENT) {
            if (xml.eventType == XmlResourceParser.START_TAG && xml.name == "string") {
                val key = xml.getAttributeValue(null, "name")
                xml.next()
                val value = xml.text
                translation[key] = value
            }
            xml.next()
        }
        return translation.toMap()
    }

    private fun defaultConfiguration(): Configuration {
        val configuration = context.resources.configuration
        configuration.setLocale(Locale(DEFAULT_LANGUAGE))
        return configuration
    }

    companion object {
        private const val DEFAULT_LANGUAGE = "en"
    }
}
