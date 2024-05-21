package net.mullvad.mullvadvpn.repository

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.ProviderId
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.SetWireguardConstraintsError
import net.mullvad.mullvadvpn.model.Settings
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class RelayListFilterRepositoryTest {
    private val mockManagementService: ManagementService = mockk()

    private lateinit var relayListFilterRepository: RelayListFilterRepository

    private val settingsFlow = MutableStateFlow(mockk<Settings>(relaxed = true))

    @BeforeEach
    fun setUp() {
        every { mockManagementService.settings } returns settingsFlow
        relayListFilterRepository =
            RelayListFilterRepository(
                managementService = mockManagementService,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @Test
    fun `when settings is updated selected ownership should update`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val selectedOwnership: Constraint<Ownership> = Constraint.Only(Ownership.MullvadOwned)
        every { mockSettings.relaySettings.relayConstraints.ownership } returns selectedOwnership

        // Act, Assert
        relayListFilterRepository.selectedOwnership.test {
            assertEquals(Constraint.Any, awaitItem())
            settingsFlow.emit(mockSettings)
            assertEquals(selectedOwnership, awaitItem())
        }
    }

    @Test
    fun `when settings is updated selected providers should update`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val selectedProviders: Constraint<Providers> =
            Constraint.Only(Providers(setOf(ProviderId("Prove"))))
        every { mockSettings.relaySettings.relayConstraints.providers } returns selectedProviders

        // Act, Assert
        relayListFilterRepository.selectedProviders.test {
            assertEquals(Constraint.Any, awaitItem())
            settingsFlow.emit(mockSettings)
            assertEquals(selectedProviders, awaitItem())
        }
    }

    @Test
    fun `when successfully updating selected ownership and filter should return unit`() = runTest {
        // Arrange
        val ownership = Constraint.Any
        val providers = Constraint.Any
        coEvery { mockManagementService.setOwnershipAndProviders(ownership, providers) } returns
            Unit.right()

        // Act
        val result =
            relayListFilterRepository.updateSelectedOwnershipAndProviderFilter(ownership, providers)

        // Assert
        coVerify { mockManagementService.setOwnershipAndProviders(ownership, providers) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `when failing to update selected ownership and filter should return SetWireguardConstraintsError`() =
        runTest {
            // Arrange
            val ownership = Constraint.Any
            val providers = Constraint.Any
            val error = SetWireguardConstraintsError.Unknown(mockk())
            coEvery { mockManagementService.setOwnershipAndProviders(ownership, providers) } returns
                error.left()

            // Act
            val result =
                relayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                    ownership,
                    providers
                )

            // Assert
            coVerify { mockManagementService.setOwnershipAndProviders(ownership, providers) }
            assertEquals(error.left(), result)
        }

    @Test
    fun `when successfully updating selected ownership should return unit`() = runTest {
        // Arrange
        val ownership = Constraint.Only(Ownership.Rented)
        coEvery { mockManagementService.setOwnership(ownership) } returns Unit.right()

        // Act
        val result = relayListFilterRepository.updateSelectedOwnership(ownership)

        // Assert
        coVerify { mockManagementService.setOwnership(ownership) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `when failing to update selected ownership should return SetWireguardConstraintsError`() =
        runTest {
            // Arrange
            val ownership = Constraint.Only(Ownership.Rented)
            val error = SetWireguardConstraintsError.Unknown(mockk())
            coEvery { mockManagementService.setOwnership(ownership) } returns error.left()

            // Act
            val result = relayListFilterRepository.updateSelectedOwnership(ownership)

            // Assert
            coVerify { mockManagementService.setOwnership(ownership) }
            assertEquals(error.left(), result)
        }

    @Test
    fun `when successfully updating selected providers should return unit`() = runTest {
        // Arrange
        val providers = Constraint.Only(Providers(setOf(ProviderId("Mopp"))))
        coEvery { mockManagementService.setProviders(providers) } returns Unit.right()

        // Act
        val result = relayListFilterRepository.updateSelectedProviders(providers)

        // Assert
        coVerify { mockManagementService.setProviders(providers) }
        assertEquals(Unit.right(), result)
    }

    @Test
    fun `when failing to update selected providers should return SetWireguardConstraintsError`() =
        runTest {
            // Arrange
            val providers = Constraint.Only(Providers(setOf(ProviderId("Mopp"))))
            val error = SetWireguardConstraintsError.Unknown(mockk())
            coEvery { mockManagementService.setProviders(providers) } returns error.left()

            // Act
            val result = relayListFilterRepository.updateSelectedProviders(providers)

            // Assert
            coVerify { mockManagementService.setProviders(providers) }
            assertEquals(error.left(), result)
        }
}
