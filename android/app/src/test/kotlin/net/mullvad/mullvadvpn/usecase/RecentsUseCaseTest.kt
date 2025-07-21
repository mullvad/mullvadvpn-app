package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CityId
import net.mullvad.mullvadvpn.lib.model.CountryId
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.Before
import org.junit.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RecentsUseCaseTest {

    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase = mockk()
    private val filteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val settingsRepository: SettingsRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)

    private lateinit var useCase: RecentsUseCase

    @Before
    fun setUp() {
        every { settingsRepository.settingsUpdates } returns settingsFlow
        useCase = RecentsUseCase(
            customListsRelayItemUseCase,
            filteredRelayListUseCase,
            settingsRepository
        )
    }

    @Test
    fun `given null settings when invoke then emit null`() = runTest {
        every { customListsRelayItemUseCase.invoke(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase.invoke(any()) } returns flowOf(emptyList())

        useCase().test {
            settingsFlow.value = null
            assertNull(awaitItem())
        }
    }

    @Test
    fun `given recents disabled when invoke then emit null`() = runTest {
        val settings = mockk<Settings> {
            every { recents } returns Recents.Disabled
        }
        every { customListsRelayItemUseCase.invoke(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase.invoke(any()) } returns flowOf(emptyList())

        useCase().test {
            settingsFlow.value = settings
            assertNull(awaitItem())
        }
    }

    @Test
    fun `given null recents when invoke then emit null`() = runTest {
        val settings = mockk<Settings> {
            every { recents } returns null
        }
        every { customListsRelayItemUseCase.invoke(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase.invoke(any()) } returns flowOf(emptyList())

        useCase().test {
            settingsFlow.value = settings
            assertNull(awaitItem())
        }
    }

    @Test
    fun `given recents enabled but empty when invoke then emit empty list`() = runTest {
        val settings = mockk<Settings> {
            every { recents } returns Recents.Enabled(emptyList())
        }
        every { customListsRelayItemUseCase.invoke(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase.invoke(any()) } returns flowOf(emptyList())

        useCase().test {
            settingsFlow.value = settings
            assertEquals(emptyList(), awaitItem())
        }
    }

    @Test
    fun `given recents enabled when invoke then emit hops`() = runTest {
        // Test data
        val swedenId = CountryId.fromCountryCode("se")
        val stockholmId = CityId.fromCityCode("sto", swedenId)
        val sweden = RelayItem.Location.Country(
            id = swedenId,
            name = "Sweden",
            cities = listOf(
                RelayItem.Location.City(
                    id = stockholmId,
                    name = "Stockholm",
                    servers = emptyList()
                )
            )
        )

        val norwayId = CountryId.fromCountryCode("no")
        val norway = RelayItem.Location.Country(
            id = norwayId,
            name = "Norway",
            cities = emptyList()
        )

        val entryCustomListId = CustomListId.generate()
        val entryCustomList = RelayItem.CustomList(
            id = entryCustomListId,
            name = CustomListName.from("My entry list").getOrThrow()
        )

        val singleHopRecent = Recent.Singlehop(stockholmId)
        val multiHopRecent = Recent.Multihop(entry = entryCustomListId, exit = norwayId)
        val missingRecent = Recent.Singlehop(CityId.fromCityCode("xxx", CountryId.fromCountryCode("xx")))

        val settings = mockk<Settings> {
            every { recents } returns Recents.Enabled(listOf(singleHopRecent, multiHopRecent, missingRecent))
        }

        every { customListsRelayItemUseCase(RelayListType.ENTRY) } returns flowOf(listOf(entryCustomList))
        every { customListsRelayItemUseCase(RelayListType.EXIT) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(RelayListType.ENTRY) } returns flowOf(listOf(sweden, norway))
        every { filteredRelayListUseCase(RelayListType.EXIT) } returns flowOf(listOf(sweden, norway))

        useCase().test {
            settingsFlow.value = settings
            val hops = awaitItem()

            val stockholmCity = sweden.cities.first()

            val expectedHops = listOf(
                Hop.Single(stockholmCity),
                Hop.Multi(entryCustomList, norway)
            )
            assertEquals(expectedHops, hops)
        }
    }
}
