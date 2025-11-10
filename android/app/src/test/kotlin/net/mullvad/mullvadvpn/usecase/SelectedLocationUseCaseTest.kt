package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class SelectedLocationUseCaseTest {
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()

    private val selectedLocation = MutableStateFlow<Constraint<RelayItemId>>(Constraint.Any)
    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))

    private lateinit var selectLocationUseCase: SelectedLocationUseCase

    @BeforeEach
    fun setup() {
        every { mockRelayListRepository.selectedLocation } returns selectedLocation
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints

        selectLocationUseCase =
            SelectedLocationUseCase(
                relayListRepository = mockRelayListRepository,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
            )
    }

    @Test
    fun `when wireguard constraints is multihop enabled should return Multiple`() = runTest {
        // Arrange
        val entryLocation: Constraint<RelayItemId> = Constraint.Only(GeoLocationId.Country("se"))
        val exitLocation = Constraint.Only(GeoLocationId.Country("us"))
        wireguardConstraints.value =
            WireguardConstraints(
                isMultihopEnabled = true,
                entryLocation = entryLocation,
                port = Constraint.Any,
                ipVersion = Constraint.Any,
            )
        selectedLocation.value = exitLocation

        // Act, Assert
        selectLocationUseCase().test {
            assertEquals(RelayItemSelection.Multiple(entryLocation, exitLocation), awaitItem())
        }
    }

    @Test
    fun `when wireguard constraints is multihop disabled should return Single`() = runTest {
        // Arrange
        val exitLocation = Constraint.Only(GeoLocationId.Country("us"))
        selectedLocation.value = exitLocation

        // Act, Assert
        selectLocationUseCase().test {
            assertEquals(RelayItemSelection.Single(exitLocation), awaitItem())
        }
    }
}
