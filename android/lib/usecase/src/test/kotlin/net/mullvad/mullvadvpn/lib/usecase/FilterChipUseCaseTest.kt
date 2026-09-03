package net.mullvad.mullvadvpn.lib.usecase

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksObfuscationSettings
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class FilterChipUseCaseTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockProviderToOwnershipsUseCase: ProviderToOwnershipsUseCase = mockk()
    private val mockMultihopInEffectUseCase: MultihopInEffectUseCase = mockk()
    private val mockSettingRepository: SettingsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()

    private val shadowsocksPortRange = MutableStateFlow<List<PortRange>>(emptyList())
    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)
    private val providerToOwnerships = MutableStateFlow<Map<ProviderId, Set<Ownership>>>(emptyMap())
    private val multihopActive = MutableStateFlow(MultihopInEffectStatus.WhenNeededInEffect)
    private val settings =
        MutableStateFlow<Settings>(
            mockk(relaxed = true) {
                every { obfuscationSettings.shadowsocks } returns
                    ShadowsocksObfuscationSettings(Constraint.Any)
            }
        )

    private lateinit var filterChipUseCase: FilterChipUseCase

    @BeforeEach
    fun setUp() {
        every { mockRelayListRepository.shadowsocksPortRanges } returns shadowsocksPortRange
        every { mockRelayListFilterRepository.selectedExitOwnership } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedExitProviders } returns selectedProviders
        every { mockRelayListFilterRepository.selectedOwnership(any()) } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedProviders(any()) } returns selectedProviders
        every { mockProviderToOwnershipsUseCase() } returns providerToOwnerships
        every { mockMultihopInEffectUseCase() } returns multihopActive
        every { mockSettingRepository.settingsUpdates } returns settings

        filterChipUseCase =
            FilterChipUseCase(
                relayListFilterRepository = mockRelayListFilterRepository,
                providerToOwnershipsUseCase = mockProviderToOwnershipsUseCase,
                settingsRepository = mockSettingRepository,
                multihopInEffectUseCase = mockMultihopInEffectUseCase,
                relayListRepository = mockRelayListRepository,
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
    fun `when Daita is enabled and multihop is disabled should return Daita filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { this@mockk.tunnelOptions.daitaSettings.enabled } returns true
                    every { obfuscationSettings.shadowsocks } returns
                        ShadowsocksObfuscationSettings(Constraint.Any)
                }

            filterChipUseCase(RelayListType.Single).test {
                assertLists(listOf(FilterChip.Daita), awaitItem())
            }
        }

    @Test
    fun `when Daita is enabled and relay list type is entry and multihop is always on should return Daita filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.daitaSettings.enabled } returns true
                    every { obfuscationSettings.shadowsocks } returns
                        ShadowsocksObfuscationSettings(Constraint.Any)
                }

            multihopActive.value = MultihopInEffectStatus.AlwaysOnInEffect

            filterChipUseCase(RelayListType.Multihop(RelayHopType.ENTRY)).test {
                assertLists(listOf(FilterChip.Daita), awaitItem())
            }
        }

    @Test
    fun `when Shadowsocks is enabled and port is outside of standard range should return Shadowsocks filter chip`() =
        runTest {
            // Arrange
            shadowsocksPortRange.value = listOf(PortRange(51900..51949))
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { obfuscationSettings.selectedObfuscationMode } returns
                        ObfuscationMode.Shadowsocks
                    every { obfuscationSettings.shadowsocks } returns
                        ShadowsocksObfuscationSettings(Constraint.Only(Port(50000)))
                }

            filterChipUseCase(RelayListType.Single).test {
                assertLists(listOf(FilterChip.Shadowsocks(Port(50000))), awaitItem())
            }
        }

    @Test
    fun `when Shadowsocks is enabled and port is inside the standard range should not return Shadowsocks filter chip`() =
        runTest {
            // Arrange
            shadowsocksPortRange.value = listOf(PortRange(51900..51949))
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { obfuscationSettings.selectedObfuscationMode } returns
                        ObfuscationMode.Shadowsocks
                    every { obfuscationSettings.shadowsocks } returns
                        ShadowsocksObfuscationSettings(Constraint.Only(Port(51920)))
                }

            filterChipUseCase(RelayListType.Single).test { assertLists(emptyList(), awaitItem()) }
        }

    @Test
    fun `when multihop when needed is enabled and relay list type is entry should return no filter chip`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.daitaSettings.enabled } returns true
                }

            selectedOwnership.value = Constraint.Only(Ownership.MullvadOwned)

            multihopActive.value = MultihopInEffectStatus.WhenNeededInEffect

            filterChipUseCase(RelayListType.Multihop(RelayHopType.ENTRY)).test {
                assertLists(emptyList(), awaitItem())
            }
        }

    @Test
    fun `when multihop when needed is enabled and relay list type is exit should return ownership and providers filters but not daita filter`() =
        runTest {
            // Arrange
            settings.value =
                mockk<Settings>(relaxed = true) {
                    every { tunnelOptions.daitaSettings.enabled } returns true
                    every { obfuscationSettings.shadowsocks } returns
                        ShadowsocksObfuscationSettings(Constraint.Any)
                }

            val expectedProviders = setOf(ProviderId("1"), ProviderId("2"))
            selectedProviders.value = Constraint.Only(expectedProviders)
            providerToOwnerships.value =
                mapOf(
                    ProviderId("1") to setOf(Ownership.MullvadOwned),
                    ProviderId("2") to setOf(Ownership.Rented),
                )

            selectedOwnership.value = Constraint.Only(Ownership.MullvadOwned)

            multihopActive.value = MultihopInEffectStatus.WhenNeededInEffect

            filterChipUseCase(RelayListType.Multihop(RelayHopType.EXIT)).test {
                assertLists(
                    listOf(
                        FilterChip.Ownership(Ownership.MullvadOwned),
                        FilterChip.Provider(1),
                    ),
                    awaitItem(),
                )
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
