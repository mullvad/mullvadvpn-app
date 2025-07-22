package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recent
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
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
        settingsFlow.value = null
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        useCase().test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents disabled when invoke then emit null`() = runTest {
        settingsFlow.value = mockk<Settings> { every { recents } returns Recents.Disabled }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        useCase().test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents enabled but empty when invoke then emit empty list`() = runTest {
        settingsFlow.value =
            mockk<Settings> { every { recents } returns Recents.Enabled(emptyList()) }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        useCase().test { assertEquals(emptyList(), awaitItem()) }
    }

    @Test
    fun `given recents enabled when invoke then emit hops based on the relay item filters`() =
        runTest {
            val swedenId = GeoLocationId.Country("se")
            val stockholmId = GeoLocationId.City(swedenId, "sto")
            val sweden =
                RelayItem.Location.Country(
                    id = swedenId,
                    name = "Sweden",
                    cities =
                        listOf(
                            RelayItem.Location.City(
                                id = stockholmId,
                                name = "Stockholm",
                                relays = emptyList(),
                            )
                        ),
                )

            val norwayId = GeoLocationId.Country("no")
            val norway =
                RelayItem.Location.Country(id = norwayId, name = "Norway", cities = emptyList())

            val entryCustomListId = CustomListId("custom")
            val customList =
                CustomList(
                    id = entryCustomListId,
                    name = CustomListName.fromString("Custom"),
                    locations = listOf(swedenId, norwayId),
                )
            val entryCustomList =
                RelayItem.CustomList(customList = customList, locations = emptyList())

            val singleHopRecent = Recent.Singlehop(stockholmId)
            val multiHopRecent = Recent.Multihop(entry = entryCustomListId, exit = norwayId)
            val filteredOutRecent =
                Recent.Singlehop(
                    GeoLocationId.City(country = GeoLocationId.Country("xx"), code = "xx-xxx-xx")
                )

            settingsFlow.value =
                mockk<Settings> {
                    every { recents } returns
                        Recents.Enabled(listOf(singleHopRecent, multiHopRecent, filteredOutRecent))
                }

            every { customListsRelayItemUseCase(RelayListType.ENTRY) } returns
                flowOf(listOf(entryCustomList))
            every { customListsRelayItemUseCase(RelayListType.EXIT) } returns flowOf(emptyList())
            every { filteredRelayListUseCase(RelayListType.ENTRY) } returns
                flowOf(listOf(sweden, norway))
            every { filteredRelayListUseCase(RelayListType.EXIT) } returns
                flowOf(listOf(sweden, norway))

            useCase().test {
                val hops = awaitItem()

                val stockholmCity = sweden.cities.first()

                val expectedHops =
                    listOf(Hop.Single(stockholmCity), Hop.Multi(entryCustomList, norway))
                assertEquals(expectedHops, hops)
            }
        }
}
