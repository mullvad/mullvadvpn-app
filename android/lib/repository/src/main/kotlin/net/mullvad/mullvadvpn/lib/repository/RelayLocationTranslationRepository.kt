package net.mullvad.mullvadvpn.lib.repository

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
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext

typealias Translations = Map<String, String>

class RelayLocationTranslationRepository(
    val context: Context,
    val localeRepository: LocaleRepository,
    externalScope: CoroutineScope = MainScope(),
    val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val translations: StateFlow<Translations> =
        localeRepository.currentLocale
            .filterNotNull()
            .map { loadTranslations(it) }
            .stateIn(externalScope, SharingStarted.Eagerly, emptyMap())

    private suspend fun loadTranslations(locale: Locale): Translations =
        withContext(dispatcher) {
            Logger.d("Updating translations based on $locale")
            if (locale.language == DEFAULT_LANGUAGE) emptyMap()
            else {
                // Load current translations
                val xml = context.resources.getXml(R.xml.relay_locations)
                xml.loadRelayTranslation()
            }
        }

    private fun XmlResourceParser.loadRelayTranslation(): Map<String, String> {
        val translation = mutableMapOf<String, String>()
        while (this.eventType != XmlResourceParser.END_DOCUMENT) {
            if (this.eventType == XmlResourceParser.START_TAG && this.name == "string") {
                val key = this.getAttributeValue(null, "name")
                val value = this.nextText()
                translation[key] = value
            }
            this.next()
        }
        return translation.toMap()
    }

    companion object {
        private const val DEFAULT_LANGUAGE = "en"
    }
}
