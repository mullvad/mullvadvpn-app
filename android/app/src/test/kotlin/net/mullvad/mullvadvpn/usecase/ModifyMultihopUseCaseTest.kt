package net.mullvad.mullvadvpn.usecase

import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled
import net.mullvad.mullvadvpn.util.wireguardConstraints
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.assertNotNull

class ModifyMultihopUseCaseTest {
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockCustomListRepository: CustomListsRepository = mockk(relaxed = true)
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings>(mockk())

    private val modifyMultihopUseCase =
        ModifyMultihopUseCase(
            relayListRepository = mockRelayListRepository,
            settingsRepository = mockSettingsRepository,
            customListsRepository = mockCustomListRepository,
            wireguardConstraintsRepository = mockWireguardConstraintsRepository,
        )

    @BeforeEach
    fun setUp() {
        mockkStatic(SETTINGS_UTIL)
        every { any<Settings>().isDaitaEnabled() } returns false
        every { any<Settings>().isDaitaDirectOnly() } returns false
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow
    }

    @Test
    fun `when changing entry and exit is the same should throw error`() = runTest {
        // Arrange
        val mockRelayItemId: GeoLocationId.Hostname = mockk()
        val mockRelayItem: RelayItem.Location.Relay = mockk()
        val mockSettings: Settings = mockk()
        every { mockSettings.relaySettings.relayConstraints.location } returns
            Constraint.Only(mockRelayItemId)
        every { mockRelayItem.id } returns mockRelayItemId
        every { mockRelayItem.active } returns true
        val change = MultihopChange.Entry(mockRelayItem)

        // Act
        settingsFlow.value = mockSettings
        val error = modifyMultihopUseCase(change = change).leftOrNull()

        // Assert
        assertInstanceOf<ModifyMultihopError.EntrySameAsExit>(error)
        assertEquals(error.relayItem.id, mockRelayItemId)
    }

    @Test
    fun `when changing exit and entry is the same should throw error`() = runTest {
        // Arrange
        val mockRelayItemId: GeoLocationId.Hostname = mockk()
        val mockRelayItem: RelayItem.Location.Relay = mockk()
        val mockSettings: Settings = mockk()
        every {
            mockSettings.relaySettings.relayConstraints.wireguardConstraints.entryLocation
        } returns Constraint.Only(mockRelayItemId)
        every { mockRelayItem.id } returns mockRelayItemId
        every { mockRelayItem.active } returns true
        val change = MultihopChange.Exit(mockRelayItem)

        // Act
        settingsFlow.value = mockSettings
        val error = modifyMultihopUseCase(change = change).leftOrNull()

        // Assert
        assertInstanceOf<ModifyMultihopError.EntrySameAsExit>(error)
        assertEquals(error.relayItem.id, mockRelayItemId)
    }

    @Test
    fun `when changing entry and exit is the same but daita is enabled without direct only should not throw error`() =
        runTest {
            // Arrange
            val mockRelayItemId: GeoLocationId.Hostname = mockk()
            val mockRelayItem: RelayItem.Location.Relay = mockk()
            val mockSettings: Settings = mockk()
            every { mockSettings.relaySettings.relayConstraints.location } returns
                Constraint.Only(mockRelayItemId)
            every { mockRelayItem.id } returns mockRelayItemId
            every { mockRelayItem.active } returns true
            every { mockSettings.isDaitaEnabled() } returns true
            coEvery { mockWireguardConstraintsRepository.setEntryLocation(mockRelayItemId) } returns
                Unit.right()
            val change = MultihopChange.Entry(mockRelayItem)

            // Act
            settingsFlow.value = mockSettings
            val result = modifyMultihopUseCase(change = change).getOrNull()

            // Assert
            coVerify { mockWireguardConstraintsRepository.setEntryLocation(mockRelayItemId) }
            assertNotNull(result)
        }

    @Test
    fun `when relay item is invalid should throw error`() = runTest {
        // Arrange
        val mockRelayItemId: GeoLocationId.Hostname = mockk()
        val mockRelayItem: RelayItem.Location.Relay = mockk()
        every { mockRelayItem.id } returns mockRelayItemId
        every { mockRelayItem.active } returns false
        val change = MultihopChange.Entry(mockRelayItem)

        // Act
        val error = modifyMultihopUseCase(change = change).leftOrNull()

        // Assert
        assertInstanceOf<ModifyMultihopError.RelayItemInactive>(error)
        assertEquals(error.relayItem.id, mockRelayItemId)
    }

    companion object {
        const val SETTINGS_UTIL = "net.mullvad.mullvadvpn.util.SettingsKt"
    }
}
