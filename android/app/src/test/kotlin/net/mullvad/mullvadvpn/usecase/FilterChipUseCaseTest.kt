package net.mullvad.mullvadvpn.usecase

import androidx.compose.material3.FilterChip
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class FilterChipUseCaseTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockAvailableProvidersUseCase: AvailableProvidersUseCase = mockk()
    private val mockSettingRepository: SettingsRepository = mockk()

    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)
    private val availableProviders = MutableStateFlow<List<Provider>>(emptyList())
    private val settings = MutableStateFlow<Settings>(mockk(relaxed = true))

    private lateinit var filterChipUseCase: FilterChipUseCase

    @BeforeEach
    fun setUp() {
        every { mockRelayListFilterRepository.selectedOwnership } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedProviders } returns selectedProviders
        every { mockAvailableProvidersUseCase() } returns availableProviders
        every { mockSettingRepository.settingsUpdates } returns settings

        filterChipUseCase =
            FilterChipUseCase(
                relayListFilterRepository = mockRelayListFilterRepository,
                availableProvidersUseCase = mockAvailableProvidersUseCase,
                settingsRepository = mockSettingRepository,
            )
    }

    @Test
    fun `when no filters are applied should return empty list`() = runTest {
        filterChipUseCase().test { assertLists(emptyList(), awaitItem()) }
    }

    @Test
    fun `when ownership filter is applied should return correct ownership`() = runTest {
        // Arrange
        val expectedOwnership = Ownership.MullvadOwned
        selectedOwnership.value = Constraint.Only(expectedOwnership)

        filterChipUseCase().test {
            assertLists(listOf(FilterChip.Ownership(expectedOwnership)), awaitItem())
        }
    }

    @Test
    fun `when provider filter is applied should return correct number of providers`() = runTest {
        // Arrange
        val expectedProviders = Providers(providers = setOf(ProviderId("1"), ProviderId("2")))
        selectedProviders.value = Constraint.Only(expectedProviders)
        availableProviders.value =
            listOf(
                Provider(ProviderId("1"), Ownership.MullvadOwned),
                Provider(ProviderId("2"), Ownership.Rented),
            )

        filterChipUseCase().test { assertLists(listOf(FilterChip.Provider(2)), awaitItem()) }
    }

    @Test
    fun `when provider and ownership filter is applied should return correct filter chips`() =
        runTest {
            // Arrange
            val expectedProviders = Providers(providers = setOf(ProviderId("1")))
            val expectedOwnership = Ownership.MullvadOwned
            selectedProviders.value = Constraint.Only(expectedProviders)
            selectedOwnership.value = Constraint.Only(expectedOwnership)
            availableProviders.value =
                listOf(
                    Provider(ProviderId("1"), Ownership.MullvadOwned),
                    Provider(ProviderId("2"), Ownership.Rented),
                )

            filterChipUseCase().test {
                assertLists(
                    listOf(FilterChip.Ownership(expectedOwnership), FilterChip.Provider(1)),
                    awaitItem(),
                )
            }
        }

    @Test
    fun `when Daita is enabled should return Daita filter chip`() = runTest {
        // Arrange
        settings.value = mockk(relaxed = true) { every { isDaitaEnabled() } returns true }

        filterChipUseCase().test { assertLists(listOf(FilterChip.Daita), awaitItem()) }
    }
}
