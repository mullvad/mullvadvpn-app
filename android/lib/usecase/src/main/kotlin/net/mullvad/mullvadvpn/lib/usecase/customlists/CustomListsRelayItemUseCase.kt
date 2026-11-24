package net.mullvad.mullvadvpn.lib.usecase.customlists

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.common.util.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.LanguageRepository
import net.mullvad.mullvadvpn.lib.repository.LocaleRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository

class CustomListsRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository,
    private val languageRepository: LanguageRepository,
    private val localeRepository: LocaleRepository,
) {

    operator fun invoke() =
        combine(
            customListsRepository.customLists,
            relayListRepository.relayList,
            localeRepository.currentLocale.filterNotNull(),
        ) { customLists, relayList, currentLocale ->
            customLists
                ?.map {
                    it.toRelayItemCustomList(
                        relayCountries = relayList,
                        locale = languageRepository.getSupportedLocaleOrEnglish(currentLocale),
                    )
                }
                .orEmpty()
        }
}
