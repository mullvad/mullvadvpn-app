package net.mullvad.mullvadvpn.feature.language.impl

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.ui.resource.R

class LanguageViewModel(
    private val languageRepository: LanguageRepository,
    private val resources: Resources,
) : ViewModel() {

    private val supportedLocales = languageRepository.getSupportedLocales()
    private val selectedLocale = MutableStateFlow(languageRepository.getAppLocale())

    val uiState: StateFlow<Lc<Unit, LanguageUiState>> =
        selectedLocale
            .map { selected ->
                LanguageUiState(
                        languages =
                            buildList {
                                add(
                                    LanguageItem(
                                        locale = null,
                                        displayName = resources.getString(R.string.system_default),
                                        isSelected = selected == null,
                                    )
                                )
                                supportedLocales.forEach { locale ->
                                    add(
                                        LanguageItem(
                                            locale = locale,
                                            displayName =
                                                locale.getDisplayName(locale).replaceFirstChar {
                                                    if (it.isLowerCase()) it.titlecase()
                                                    else it.toString()
                                                },
                                            isSelected =
                                                selected != null &&
                                                    locale.toLanguageTag() ==
                                                        selected.toLanguageTag(),
                                        )
                                    )
                                }
                            }
                    )
                    .toLc<Unit, LanguageUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun setLanguage(locale: Locale?) {
        selectedLocale.value = locale
        languageRepository.setAppLocale(locale)
    }
}
