package net.mullvad.mullvadvpn.feature.language.impl

import java.util.Locale

sealed interface LanguageItem {
    val isSelected: Boolean
    val locale: Locale?
    data class Language(override val locale: Locale, val displayName: String,
                        override val isSelected: Boolean): LanguageItem {
        override val key: String = locale.toLanguageTag()
    }
    data class SystemDefault(override val isSelected: Boolean): LanguageItem {
        override val key: String = "system_default"
        override val locale: Locale? = null
    }

    val key: String
}
