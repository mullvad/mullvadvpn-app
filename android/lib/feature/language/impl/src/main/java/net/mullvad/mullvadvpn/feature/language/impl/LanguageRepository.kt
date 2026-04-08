package net.mullvad.mullvadvpn.feature.language.impl

import android.app.LocaleConfig
import android.app.LocaleManager
import android.content.Context
import android.os.LocaleList
import androidx.annotation.RequiresApi
import java.util.Locale

@RequiresApi(android.os.Build.VERSION_CODES.TIRAMISU)
class LanguageRepository(private val context: Context) {

    private val localeManager: LocaleManager =
        context.getSystemService(LocaleManager::class.java)

    fun getSupportedLocales(): List<Locale> {
        val localeList = LocaleConfig(context).supportedLocales ?: return emptyList()
        return buildList {
                for (i in 0 until localeList.size()) {
                    add(localeList.get(i))
                }
            }
            .sortedBy { it.getDisplayName(it).lowercase() }
    }

    fun getAppLocale(): Locale? {
        val locales = localeManager.applicationLocales
        return if (locales.isEmpty) null else locales.get(0)
    }

    fun setAppLocale(locale: Locale?) {
        localeManager.applicationLocales =
            if (locale == null) {
                LocaleList.getEmptyLocaleList()
            } else {
                LocaleList.forLanguageTags(locale.toLanguageTag())
            }
    }
}


