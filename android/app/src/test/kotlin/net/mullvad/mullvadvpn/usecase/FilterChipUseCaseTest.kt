package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class FilterChipUseCaseTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockProviderToOwnershipsUseCase: ProviderToOwnershipsUseCase = mockk()
    private val mockSettingRepository: SettingsRepository = mockk()

    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)
    private val providerToOwnerships = MutableStateFlow<Map<ProviderId, Set<Ownership>>>(emptyMap())
    private val settings = MutableStateFlow<Settings>(mockk(relaxed = true))

    private lateinit var filterChipUseCase: FilterChipUseCase

    @BeforeEach
    fun setUp() {
        every { mockRelayListFilterRepository.selectedOwnership(any()) } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedOwnership(any()) } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedProviders(any()) } returns selectedProviders
        every { mockRelayListFilterRepository.selectedProviders(any()) } returns selectedProviders
        every { mockProviderToOwnershipsUseCase() } returns providerToOwnerships
        every { mockSettingRepository.settingsUpdates } returns settings

        filterChipUseCase =
            FilterChipUseCase(
                relayListFilterRepository = mockRelayListFilterRepository,
                providerToOwnershipsUseCase = mockProviderToOwnershipsUseCase,
                settingsRepository = mockSettingRepository,
            )
    }

    @Test
    fun `when no filters are applied should return empty list`() = runTest {
        filterChipUseCase(RelayListType.Single).test { assertLists(emptyList(), awaitItem()) }
    }

    @Test
    fun `when ownership filter is applied should return correct ownership`() = runTest {
        // Arrange
        val expectedOwnership = Ownership.MullvadOwned
        selectedOwnership.value = Constraint.Only(expectedOwnership)

        filterChipUseCase(RelayListType.Single).test {
            assertLists(listOf(FilterChip.Ownership(expectedOwnership)), awaitItem())
        }
    }

    @Test
    fun `when provider filter is applied should return correct number of providers`() = runTest {
        // Arrange
        val expectedProviders = setOf(ProviderId("1"), ProviderId("2"))
        selectedProviders.value = Constraint.Only(expectedProviders)
        providerToOwnerships.value =
            mapOf(
                ProviderId("1") to setOf(Ownership.MullvadOwned),
                ProviderId("2") to setOf(Ownership.Rented),
            )

        filterChipUseCase(RelayListType.Single).test {
            assertLists(listOf(FilterChip.Provider(2)), awaitItem())
        }
    }

    @Test
    fun `when provider and ownership filter is applied should return correct filter chips`() =
        runTest {
            // Arrange
            val expectedProviders = setOf(ProviderId("1"))
            val expectedOwnership = Ownership.MullvadOwned
            selectedProviders.value = Constraint.Only(expectedProviders)
            selectedOwnership.value = Constraint.Only(expectedOwnership)
            providerToOwnerships.value =
                mapOf(
                    ProviderId("1") to setOf(Ownership.MullvadOwned),
                    ProviderId("2") to setOf(Ownership.Rented),
                )

            filterChipUseCase(RelayListType.Single).test {
                assertLists(
                    listOf(FilterChip.Ownership(expectedOwnership), FilterChip.Provider(1)),
                    awaitItem(),
                )
            }
        }

    @Test
    fun `when Daita with direct only is enabled and multihop is disabled should return Daita filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { this@mockk.tunnelOptions.wireguard.daitaSettings.enabled } returns true
                    every { tunnelOptions.wireguard.daitaSettings.directOnly } returns true
                }

            filterChipUseCase(RelayListType.Single).test {
                assertLists(listOf(FilterChip.Daita), awaitItem())
            }
        }

    @Test
    fun `when Daita without direct only is enabled and multihop is disabled should return no filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.wireguard.daitaSettings.enabled } returns true
                    every { tunnelOptions.wireguard.daitaSettings.directOnly } returns false
                }

            filterChipUseCase(RelayListType.Single).test { assertLists(emptyList(), awaitItem()) }
        }

    @Test
    fun `when Daita with direct only is enabled and relay list type is entry should return Daita filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.wireguard.daitaSettings.enabled } returns true
                    every { tunnelOptions.wireguard.daitaSettings.directOnly } returns true
                }

            filterChipUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY)).test {
                assertLists(listOf(FilterChip.Daita), awaitItem())
            }
        }

    @Test
    fun `when Daita with direct only is enabled and relay list type is exit should return no filter`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.wireguard.daitaSettings.enabled } returns true
                    every { tunnelOptions.wireguard.daitaSettings.directOnly } returns true
                }

            filterChipUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT)).test {
                assertLists(emptyList(), awaitItem())
            }
        }

    @Test
    fun `when Daita without direct only is enabled and relay list type is exit should return no filter`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.wireguard.daitaSettings.enabled } returns true
                    every { tunnelOptions.wireguard.daitaSettings.directOnly } returns false
                }

            filterChipUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT)).test {
                assertLists(emptyList(), awaitItem())
            }
        }

    @Test
    fun `ensure that a selected provider that is not in the provider list is still counted`() =
        runTest {
            // Arrange
            val expectedProviders = setOf(ProviderId("1"))
            val expectedOwnership = Ownership.MullvadOwned
            selectedProviders.value = Constraint.Only(expectedProviders)
            selectedOwnership.value = Constraint.Only(expectedOwnership)
            providerToOwnerships.value =
                mapOf(
                    ProviderId("2") to setOf(Ownership.MullvadOwned),
                    ProviderId("3") to setOf(Ownership.Rented),
                )

            // Act, Assert
            filterChipUseCase(RelayListType.Single).test {
                assertLists(
                    listOf(FilterChip.Ownership(expectedOwnership), FilterChip.Provider(1)),
                    awaitItem(),
                )
            }
        }
}
