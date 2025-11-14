package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.HopSelection
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
        // Arrange
        settingsFlow.value = null
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        // Act, Assert
        useCase(RelayListType.Single).test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents disabled when invoke then emit null`() = runTest {
        // Arrange
        settingsFlow.value = mockk<Settings> { every { recents } returns Recents.Disabled }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        // Act, Assert
        useCase(RelayListType.Single).test { assertNull(awaitItem()) }
    }

    @Test
    fun `given recents enabled but empty when invoke then emit empty list`() = runTest {
        // Arrange
        settingsFlow.value =
            mockk<Settings> { every { recents } returns Recents.Enabled(emptyList()) }
        every { customListsRelayItemUseCase(any()) } returns flowOf(emptyList())
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

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
        val recent = Recent.Singlehop(location = id)
        settingsFlow.value =
            mockk<Settings> { every { recents } returns Recents.Enabled(listOf(recent)) }
        every { customListsRelayItemUseCase(any()) } returns flowOf(listOf(customList))
        every { filteredRelayListUseCase(any()) } returns flowOf(emptyList())

        useCase(RelayListType.Single).test { assertEquals(emptyList(), awaitItem()) }
    }

    @Test
    fun `given recents enabled when invoke then emit hops based on the relay item filters`() =
        runTest {
            val singlehopRecent = Recent.Singlehop(STOCKHOLM_ID)
            val filteredOutRecent =
                Recent.Singlehop(
                    GeoLocationId.City(country = GeoLocationId.Country("xx"), code = "xx-xxx-xx")
                )

            settingsFlow.value =
                mockk<Settings> {
                    every { recents } returns
                        Recents.Enabled(listOf(singlehopRecent, filteredOutRecent))
                }

            every { customListsRelayItemUseCase(RelayListType.Single) } returns flowOf(emptyList())
            every { filteredRelayListUseCase(RelayListType.Single) } returns
                flowOf(listOf(SWEDEN, NORWAY))

            useCase(RelayListType.Single).test {
                val hops = awaitItem()

                val expectedHops = listOf(HopSelection.Single(STOCKHOLM))
                assertLists(expectedHops, hops!!)
            }
        }

    @Test
    fun `given multihop true should filter out singlehop recents`() = runTest {
        val singlehopRecent = Recent.Singlehop(STOCKHOLM_ID)
        val multihopRecent = Recent.Multihop(entry = CUSTOM_LIST_ID, exit = NORWAY_ID)

        settingsFlow.value =
            mockk<Settings> {
                every { recents } returns Recents.Enabled(listOf(singlehopRecent, multihopRecent))
            }

        every {
            customListsRelayItemUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY))
        } returns flowOf(listOf(CUSTOM_LIST_SWE_NO))
        every {
            customListsRelayItemUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT))
        } returns flowOf(emptyList())
        every {
            filteredRelayListUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY))
        } returns flowOf(listOf(SWEDEN, NORWAY))
        every {
            filteredRelayListUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT))
        } returns flowOf(listOf(SWEDEN, NORWAY))

        useCase(RelayListType.Multihop(MultihopRelayListType.ENTRY)).test {
            val hops = awaitItem()

            val expectedHops = listOf(HopSelection.Single(CUSTOM_LIST_SWE_NO))
            assertLists(expectedHops, hops!!)
        }
    }

    companion object {
        private val SWEDEN_ID = GeoLocationId.Country("se")
        private val STOCKHOLM_ID = GeoLocationId.City(SWEDEN_ID, "sto")
        private val STOCKHOLM =
            RelayItem.Location.City(id = STOCKHOLM_ID, name = "Stockholm", relays = emptyList())
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
