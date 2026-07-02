package net.mullvad.mullvadvpn.lib.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class SelectedLocationUseCaseTest {
    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()

    private val multihopInEffectUseCase =
        MultihopInEffectUseCase(
            connectionProxy = mockConnectionProxy,
            wireguardConstraintsRepository = mockWireguardConstraintsRepository,
        )

    private val tunnelState =
        MutableStateFlow<TunnelState>(
            TunnelState.Connected(
                endpoint = mockk(),
                location = mockk(),
                featureIndicators = listOf(FeatureIndicator.MULTIHOP_AUTO),
            )
        )

    private val selectedLocation = MutableStateFlow<Constraint<RelayItemId>>(Constraint.Any)
    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))

    private lateinit var selectLocationUseCase: SelectedLocationUseCase

    @BeforeEach
    fun setup() {
        every { mockRelayListRepository.selectedLocation } returns selectedLocation
        every { mockConnectionProxy.tunnelState } returns tunnelState
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints

        selectLocationUseCase =
            SelectedLocationUseCase(
                relayListRepository = mockRelayListRepository,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                multihopInEffectUseCase = multihopInEffectUseCase,
            )
    }

    @Test
    fun `when wireguard constraints is multihop always should return Multiple`() = runTest {
        // Arrange
        val entryLocation: Constraint<RelayItemId> = Constraint.Only(GeoLocationId.Country("se"))
        val exitLocation = Constraint.Only(GeoLocationId.Country("us"))
        wireguardConstraints.value =
            WireguardConstraints(
                multihop = MultihopMode.ALWAYS,
                entryLocation = entryLocation,
                ipVersion = Constraint.Any,
                entryOwnership = Constraint.Any,
                entryProviders = Constraint.Any,
            )
        selectedLocation.value = exitLocation

        // Act, Assert
        selectLocationUseCase().test {
            assertEquals(RelayItemSelection.Multiple(entryLocation, exitLocation), awaitItem())
        }
    }

    @Test
    fun `when wireguard constraints is multihop when needed should return Multiple`() = runTest {
        // Arrange
        val entryLocation: Constraint<RelayItemId> = Constraint.Any
        val exitLocation = Constraint.Only(GeoLocationId.Country("us"))
        wireguardConstraints.value =
            WireguardConstraints(
                multihop = MultihopMode.WHEN_NEEDED,
                entryLocation = entryLocation,
                ipVersion = Constraint.Any,
                entryOwnership = Constraint.Any,
                entryProviders = Constraint.Any,
            )
        selectedLocation.value = exitLocation

        // Act, Assert
        selectLocationUseCase().test {
            assertEquals(RelayItemSelection.Multiple(entryLocation, exitLocation), awaitItem())
        }
    }

    @Test
    fun `when wireguard constraints is multihop never should return Single`() = runTest {
        // Arrange
        val exitLocation = Constraint.Only(GeoLocationId.Country("us"))
        wireguardConstraints.value =
            WireguardConstraints(
                multihop = MultihopMode.NEVER,
                entryLocation = Constraint.Any,
                ipVersion = Constraint.Any,
                entryOwnership = Constraint.Any,
                entryProviders = Constraint.Any,
            )
        selectedLocation.value = exitLocation

        // Act, Assert
        selectLocationUseCase().test {
            assertEquals(RelayItemSelection.Single(exitLocation), awaitItem())
        }
    }
}
