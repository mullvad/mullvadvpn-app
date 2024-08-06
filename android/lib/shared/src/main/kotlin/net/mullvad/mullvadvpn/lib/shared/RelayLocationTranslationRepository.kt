package net.mullvad.mullvadvpn.lib.shared

import android.content.Context
import android.content.res.Configuration
import android.content.res.XmlResourceParser
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlin.also
import kotlin.collections.associate
import kotlin.collections.set
import kotlin.collections.toMap
import kotlin.to
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext

typealias Translations = Map<String, String>

class RelayLocationTranslationRepository(
    val context: Context,
    val localeRepository: LocaleRepository,
    externalScpoe: CoroutineScope,
    val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {

    val translations: StateFlow<Translations> =
        localeRepository.currentLocale
            .map { loadTranslations(it) }
            .stateIn(externalScpoe, SharingStarted.Eagerly, emptyMap())

    private val defaultTranslation: Map<String, String>

    init {
        val defaultConfiguration = defaultConfiguration()
        val confContext = context.createConfigurationContext(defaultConfiguration)
        val defaultTranslationXml = confContext.resources.getXml(R.xml.relay_locations)
        defaultTranslation = loadRelayTranslation(defaultTranslationXml)
    }

    private suspend fun loadTranslations(locale: Locale?): Translations =
        withContext(dispatcher) {
            Logger.d(
                "AppLang ${this@RelayLocationTranslationRepository}: Updating based on current locale to $locale"
            )
            if (locale == null || locale.language == DEFAULT_LANGUAGE) emptyMap()
            else {
                // Load current translations
                val xml = context.resources.getXml(R.xml.relay_locations)
                val translation = loadRelayTranslation(xml)

                translation.entries
                    .associate { (id, name) -> defaultTranslation[id]!! to name }
                    .also { Logger.d("AppLang: New translationTable: $it") }
            }
        }

    private fun loadRelayTranslation(xml: XmlResourceParser): Map<String, String> {
        val translation = mutableMapOf<String, String>()
        while (xml.eventType != XmlResourceParser.END_DOCUMENT) {
            if (xml.eventType == XmlResourceParser.START_TAG && xml.name == "string") {
                val key = xml.getAttributeValue(null, "name")
                val value = xml.nextText()
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
