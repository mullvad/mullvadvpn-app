package net.mullvad.mullvadvpn.feature.customvpn.impl.list

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.io.ByteArrayInputStream
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_PRIVATE_KEY
import net.mullvad.mullvadvpn.feature.customvpn.impl.createSampleCustomVpnConfig
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.GetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.repository.CustomTunnelsRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@OptIn(ExperimentalCoroutinesApi::class)
@ExtendWith(TestCoroutineRule::class)
class CustomTunnelsListViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockCustomTunnelsRepository: CustomTunnelsRepository = mockk()
    private val mockContentResolver: ContentResolver = mockk()
    private val uri: Uri = mockk()

    private val settingsUpdates = MutableStateFlow<Settings?>(null)
    private var importedConfigContent: String = ""

    private lateinit var viewModel: CustomTunnelsListViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns settingsUpdates
        coEvery { mockCustomTunnelsRepository.getCustomTunnelStats() } returns
            GetCustomVpnConfigError.Unknown.left()
        every { mockContentResolver.openInputStream(uri) } answers
            {
                ByteArrayInputStream(importedConfigContent.toByteArray(Charsets.UTF_8))
            }

        viewModel =
            CustomTunnelsListViewModel(
                settingsRepository = mockSettingsRepository,
                customTunnelsRepository = mockCustomTunnelsRepository,
                ioDispatcher = UnconfinedTestDispatcher(),
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        viewModel.uiState.test {
            val initial = awaitItem()
            assertIs<Lc.Loading<Unit>>(initial)
            assertEquals(Unit, initial.value)
        }
    }

    @Test
    fun `when settings updates emit with null custom vpn config uiState should emit content with null config and initial stats`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(Unit), awaitItem())

                initSettings(customVpnConfig = null)

                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelsListUiState>>(content)
                assertNull(content.value.customVpnConfig)
                assertEquals(TunnelStats(0, 0), content.value.tunnelStats)
            }
        }

    @Test
    fun `when settings updates emit with custom vpn config uiState should emit content with config and initial stats`() =
        runTest {
            // Arrange
            val sampleConfig = createSampleCustomVpnConfig()

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(Unit), awaitItem())

                initSettings(customVpnConfig = sampleConfig)

                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelsListUiState>>(content)
                assertEquals(sampleConfig, content.value.customVpnConfig)
                assertEquals(TunnelStats(0, 0), content.value.tunnelStats)
            }
        }

    @Test
    fun `when repository returns updated tunnel stats uiState should emit updated stats`() =
        runTest {
            // Arrange
            val initialStats = TunnelStats(rx = 100, tx = 200)
            val updatedStats = TunnelStats(rx = 300, tx = 400)
            coEvery { mockCustomTunnelsRepository.getCustomTunnelStats() } returns
                initialStats.right() andThen
                updatedStats.right()

            initSettings(customVpnConfig = null)

            // Act, Assert
            viewModel.uiState.test {
                val initialContent = awaitItem()
                assertIs<Lc.Content<CustomTunnelsListUiState>>(initialContent)
                assertEquals(initialStats, initialContent.value.tunnelStats)

                advanceTimeBy(1.seconds)

                val updatedContent = awaitItem()
                assertIs<Lc.Content<CustomTunnelsListUiState>>(updatedContent)
                assertEquals(updatedStats, updatedContent.value.tunnelStats)
            }
        }

    @Test
    fun `when getCustomVpnStats fails uiState should retain previous stats`() = runTest {
        // Arrange
        val initialStats = TunnelStats(rx = 100, tx = 200)
        coEvery { mockCustomTunnelsRepository.getCustomTunnelStats() } returns
            initialStats.right() andThen
            GetCustomVpnConfigError.Unknown.left()

        initSettings(customVpnConfig = null)

        // Act, Assert
        viewModel.uiState.test {
            val initialContent = awaitItem()
            assertIs<Lc.Content<CustomTunnelsListUiState>>(initialContent)
            assertEquals(initialStats, initialContent.value.tunnelStats)

            advanceTimeBy(1.seconds)

            expectNoEvents()
            assertEquals(initialStats, viewModel.uiState.value.contentOrNull()?.tunnelStats)
        }
    }

    @Test
    fun `onToggleCustomTunnel with enabled config should toggle to disabled`() = runTest {
        // Arrange
        val enabledConfig = createSampleCustomVpnConfig(enabled = true)
        val expectedConfig = enabledConfig.copy(enabled = false)
        coEvery { mockCustomTunnelsRepository.setConfig(expectedConfig) } returns Unit.right()

        // Act
        viewModel.onToggleCustomTunnel(enabledConfig)

        // Assert
        coVerify(exactly = 1) { mockCustomTunnelsRepository.setConfig(expectedConfig) }
    }

    @Test
    fun `onToggleCustomTunnel with disabled config should toggle to enabled`() = runTest {
        // Arrange
        val disabledConfig = createSampleCustomVpnConfig(enabled = false)
        val expectedConfig = disabledConfig.copy(enabled = true)
        coEvery { mockCustomTunnelsRepository.setConfig(expectedConfig) } returns Unit.right()

        // Act
        viewModel.onToggleCustomTunnel(disabledConfig)

        // Assert
        coVerify(exactly = 1) { mockCustomTunnelsRepository.setConfig(expectedConfig) }
    }

    @Test
    fun `when importConfig succeeds should emit ConfigurationSaved side effect`() = runTest {
        // Arrange
        val name = "test-tunnel"
        val configString = "[Interface]\nPrivateKey = $VALID_PRIVATE_KEY\n"
        importedConfigContent = configString
        coEvery { mockCustomTunnelsRepository.importConfig(name, configString) } returns
            Unit.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.importConfig(name, mockContentResolver, uri)
            assertEquals(CustomTunnelsListSideEffect.ConfigurationSaved, awaitItem())
        }

        coVerify(exactly = 1) { mockCustomTunnelsRepository.importConfig(name, configString) }
    }

    @Test
    fun `when importConfig fails should emit FailedToSave side effect`() = runTest {
        // Arrange
        val name = "test-tunnel"
        val configString = "invalid config content"
        importedConfigContent = configString
        val error = SetCustomVpnConfigError.Unknown(RuntimeException("Invalid configuration"))
        coEvery { mockCustomTunnelsRepository.importConfig(name, configString) } returns
            error.left()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.importConfig(name, mockContentResolver, uri)
            val sideEffect = awaitItem()
            assertIs<CustomTunnelsListSideEffect.FailedToSave>(sideEffect)
            assertEquals(error.toString(), sideEffect.reason)
        }

        coVerify(exactly = 1) { mockCustomTunnelsRepository.importConfig(name, configString) }
    }

    private fun initSettings(customVpnConfig: CustomVpnConfig? = null) {
        val mockSettings = mockk<Settings>(relaxed = true)
        every { mockSettings.customVpnConfig } returns customVpnConfig
        settingsUpdates.value = mockSettings
    }
}
