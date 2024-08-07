package net.mullvad.mullvadvpn.lib.shared

import android.content.Context
import android.content.res.XmlResourceParser
import co.touchlab.kermit.Logger
import java.util.Locale
import kotlin.collections.set
import kotlin.collections.toMap
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext

typealias Translations = Map<String, String>

class RelayLocationTranslationRepository(
    val context: Context,
    val localeRepository: LocaleRepository,
    externalScope: CoroutineScope = MainScope(),
    val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val translations: StateFlow<Translations> =
        localeRepository.currentLocale
            .map { loadTranslations(it) }
            .stateIn(externalScope, SharingStarted.Eagerly, emptyMap())

    private suspend fun loadTranslations(locale: Locale?): Translations =
        withContext(dispatcher) {
            Logger.d("Updating translations based $locale")
            if (locale == null || locale.language == DEFAULT_LANGUAGE) emptyMap()
            else {
                // Load current translations
                val xml = context.resources.getXml(R.xml.relay_locations)
                loadRelayTranslation(xml)
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

    companion object {
        private const val DEFAULT_LANGUAGE = "en"
    }
}
