package net.mullvad.mullvadvpn.lib.usecase.customlists

import kotlin.collections.map
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.common.util.relaylist.toRelayItemCustomList
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.LanguageRepository
import net.mullvad.mullvadvpn.lib.repository.LocaleRepository
import net.mullvad.mullvadvpn.lib.usecase.FilteredRelayListUseCase

class FilterCustomListsRelayItemUseCase(
    private val customListsRepository: CustomListsRepository,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val languageRepository: LanguageRepository,
    private val localeRepository: LocaleRepository,
) {

    operator fun invoke(relayListType: RelayListType) =
        combine(
            customListsRepository.customLists,
            filteredRelayListUseCase(relayListType),
            localeRepository.currentLocale.filterNotNull(),
        ) { customLists, filteredRelayList, currentLocale ->
            customLists
                ?.map {
                    it.toRelayItemCustomList(
                        relayCountries = filteredRelayList.countries,
                        locale = languageRepository.getSupportedLocaleOrEnglish(currentLocale),
                    )
                }
                .orEmpty()
        }
}
