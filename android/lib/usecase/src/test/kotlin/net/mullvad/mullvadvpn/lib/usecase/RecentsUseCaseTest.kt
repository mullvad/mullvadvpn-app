package net.mullvad.mullvadvpn.lib.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.EntryRecent
import net.mullvad.mullvadvpn.lib.model.ExitRecent
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.RecentItem
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.usecase.customlists.FilterCustomListsRelayItemUseCase
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class RecentsUseCaseTest {

    private val customListsRelayItemUseCase: FilterCustomListsRelayItemUseCase = mockk()
    private val filteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val settingsRepository: SettingsRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)

    private lateinit var useCase: RecentsUseCase

    @BeforeEach
    fun setUp() {
        every { settingsRepository.settingsUpdates } returns settingsFlow
        useCase =
            RecentsUseCase(
                customListsRelayItemUseCase,
                filteredRelayListUseCase,
                settingsRepository,
            )
    }

    @Test
    fun `given null settings when invoke then emit null`() = runTest {
        // Arrange
        settingsFlow.value = null
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(FilteredCountries())

        // Act, Assert
        useCase(RelayListType.Single).test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents disabled when invoke then emit null`() = runTest {
        // Arrange
        settingsFlow.value = mockk<Settings> { every { recents } returns Recents.Disabled }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(FilteredCountries())

        // Act, Assert
        useCase(RelayListType.Single).test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents enabled but empty when invoke then emit empty list`() = runTest {
        // Arrange
        settingsFlow.value = mockk<Settings> { every { recents } returns enabled() }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(FilteredCountries())

        // Act, Assert
        useCase(RelayListType.Single).test { assertEquals(emptyList(), awaitItem()) }
    }

    @Test
    fun `given recent custom list with no children should not emit that recent`() = runTest {
        // Arrange
        val id = CustomListId("id")
        val customList =
            RelayItem.CustomList(
                customList =
                    CustomList(
                        id = id,
                        name = CustomListName.fromString("name"),
                        locations = emptyList(),
                    ),
                locations = emptyList(),
            )
        val recent = ExitRecent(location = id)
        settingsFlow.value =
            mockk<Settings> { every { recents } returns enabled(exit = listOf(recent)) }
        every { customListsRelayItemUseCase(any()) } returns flowOf(listOf(customList))
        every { filteredRelayListUseCase(any()) } returns flowOf(FilteredCountries())

        useCase(RelayListType.Single).test { assertEquals(emptyList(), awaitItem()) }
    }

    @Test
    fun `given recents enabled when invoke then emit hops based on the relay item filters`() =
        runTest {
            val singlehopRecent = ExitRecent(STOCKHOLM_ID)
            val filteredOutRecent =
                ExitRecent(
                    GeoLocationId.City(country = GeoLocationId.Country("xx"), code = "xx-xxx-xx")
                )

            settingsFlow.value =
                mockk<Settings> {
                    every { recents } returns
                        enabled(exit = listOf(singlehopRecent, filteredOutRecent))
                }

            every { customListsRelayItemUseCase(RelayListType.Single) } returns flowOf(emptyList())
            every { filteredRelayListUseCase(RelayListType.Single) } returns
                flowOf(FilteredCountries(countries = listOf(SWEDEN, NORWAY)))

            useCase(RelayListType.Single).test {
                val recents = awaitItem()

                val expectedHops = listOf(STOCKHOLM)
                assertLists(expectedHops, recents!!.map { (it as RecentItem.Relay).item })
            }
        }

    @Test
    fun `given multihop entry should return only entry recents`() = runTest {
        val singlehopRecent = ExitRecent(STOCKHOLM_ID)
        val multihopEntry = EntryRecent.Location(CUSTOM_LIST_ID)
        val multihopExit = ExitRecent(NORWAY_ID)

        settingsFlow.value =
            mockk<Settings> {
                every { recents } returns
                    enabled(
                        entry = listOf(multihopEntry),
                        exit = listOf(singlehopRecent, multihopExit),
                    )
            }

        every { customListsRelayItemUseCase(RelayListType.Multihop(RelayHopType.ENTRY)) } returns
            flowOf(listOf(CUSTOM_LIST_SWE_NO))
        every { customListsRelayItemUseCase(RelayListType.Multihop(RelayHopType.EXIT)) } returns
            flowOf(emptyList())
        every { filteredRelayListUseCase(RelayListType.Multihop(RelayHopType.ENTRY)) } returns
            flowOf(FilteredCountries(countries = listOf(SWEDEN, NORWAY)))
        every { filteredRelayListUseCase(RelayListType.Multihop(RelayHopType.EXIT)) } returns
            flowOf(FilteredCountries(countries = listOf(SWEDEN, NORWAY)))

        useCase(RelayListType.Multihop(RelayHopType.ENTRY)).test {
            val recents = awaitItem()

            val expectedHops = listOf(CUSTOM_LIST_SWE_NO)
            assertLists(expectedHops, recents!!.map { (it as RecentItem.Relay).item })
        }
    }

    fun enabled(
        entry: List<EntryRecent> = emptyList(),
        exit: List<ExitRecent> = emptyList(),
    ): Recents.Enabled = Recents.Enabled(entry = entry, exit = exit)

    companion object {
        private val SWEDEN_ID = GeoLocationId.Country("se")
        private val STOCKHOLM_ID = GeoLocationId.City(SWEDEN_ID, "sto")
        private val STOCKHOLM =
            RelayItem.Location.City(
                id = STOCKHOLM_ID,
                name = "Stockholm",
                relays = emptyList(),
                countryName = "Sweden",
                latLong = LatLong(0f, 0f),
            )
        private val SWEDEN =
            RelayItem.Location.Country(id = SWEDEN_ID, name = "Sweden", cities = listOf(STOCKHOLM))
        private val NORWAY_ID = GeoLocationId.Country("no")
        private val NORWAY =
            RelayItem.Location.Country(id = NORWAY_ID, name = "Norway", cities = emptyList())
        private val CUSTOM_LIST_ID = CustomListId("custom")
        private val CUSTOM_LIST_SWE_NO =
            RelayItem.CustomList(
                customList =
                    CustomList(
                        id = CUSTOM_LIST_ID,
                        name = CustomListName.fromString("Custom"),
                        locations = listOf(SWEDEN_ID, NORWAY_ID),
                    ),
                locations = listOf(SWEDEN, NORWAY),
            )
    }
}
