package net.mullvad.mullvadvpn.lib.repository

import android.app.LocaleConfig
import android.app.LocaleManager
import android.content.Context
import android.os.Build
import android.os.LocaleList
import androidx.annotation.RequiresApi
import java.util.Locale
import org.xmlpull.v1.XmlPullParser

class LanguageRepository(private val context: Context) {

    @delegate:RequiresApi(Build.VERSION_CODES.TIRAMISU)
    private val localeManager: LocaleManager by lazy {
        context.getSystemService(LocaleManager::class.java)
    }

    fun getSupportedLocales(): List<Locale> {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val localeList = LocaleConfig(context).supportedLocales ?: return emptyList()
            buildList {
                for (i in 0 until localeList.size()) {
                    add(localeList.get(i))
                }
            }
                .sortedBy { it.getDisplayName(it).lowercase() }
        } else {
            buildList {
                context.resources.getXml(R.xml.locales_config).use { p ->
                    while (p.next() != XmlPullParser.END_DOCUMENT) {
                        if (p.eventType == XmlPullParser.START_TAG && p.name == "locale") {
                            add(Locale.forLanguageTag(p.getAttributeValue(0)))
                        }
                    }
                }
            }
        }
    }

    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun getAppLocale(): Locale? {
        val locales = localeManager.applicationLocales
        return if (locales.isEmpty) null else locales.get(0)
    }

    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun setAppLocale(locale: Locale?) {
        localeManager.applicationLocales =
            if (locale == null) {
                LocaleList.getEmptyLocaleList()
            } else {
                LocaleList.forLanguageTags(locale.toLanguageTag())
            }
    }

    fun getSupportedLocaleOrEnglish(locale: Locale = Locale.getDefault()): Locale {
        return if (getSupportedLocales().any { it.language == locale.language }) {
            locale
        } else {
            Locale.ENGLISH
        }
    }
}
