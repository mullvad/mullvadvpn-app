package net.mullvad.mullvadvpn.viewmodel

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import java.io.InputStream
import java.io.InputStreamReader
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.model.SettingsPatchError
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ServerIpOverridesViewModelTest {
    private lateinit var viewModel: ServerIpOverridesViewModel

    private val mockRelayOverridesRepository: RelayOverridesRepository = mockk()
    private val mockContentResolver: ContentResolver = mockk()

    private val relayOverrides = MutableStateFlow<List<RelayOverride>?>(null)

    @BeforeEach
    fun setup() {
        coEvery { mockRelayOverridesRepository.relayOverrides } returns relayOverrides

        mockkStatic(READ_TEXT)

        viewModel =
            ServerIpOverridesViewModel(
                relayOverridesRepository = mockRelayOverridesRepository,
                contentResolver = mockContentResolver
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `ensure state is loading by default`() = runTest {
        viewModel.uiState.test { assertEquals(ServerIpOverridesViewState.Loading, awaitItem()) }
    }

    @Test
    fun `when server ip overrides are empty ui state overrides should be inactive`() = runTest {
        viewModel.uiState.test {
            assertEquals(ServerIpOverridesViewState.Loading, awaitItem())
            relayOverrides.emit(emptyList())
            assertEquals(ServerIpOverridesViewState.Loaded(false), awaitItem())
        }
    }

    @Test
    fun `when import is finished we should get side effect`() = runTest {
        val mockkResult: SettingsPatchError = mockk()
        coEvery { mockRelayOverridesRepository.applySettingsPatch(TEXT_INPUT) } returns Unit.right()

        viewModel.uiSideEffect.test {
            viewModel.importText(TEXT_INPUT)
            assertEquals(ServerIpOverridesUiSideEffect.ImportResult(mockkResult), awaitItem())
        }
    }

    @Test
    fun `ensure import text invokes repository`() = runTest {
        viewModel.importText(TEXT_INPUT)

        coVerify { mockRelayOverridesRepository.applySettingsPatch(TEXT_INPUT) }
    }

    @Test
    fun `ensure import file invokes repository`() = runTest {
        val uri: Uri = mockk()

        val mockInputStream: InputStream = mockk()
        every { mockContentResolver.openInputStream(uri) } returns mockInputStream
        every { any<InputStreamReader>().readText() } returns TEXT_INPUT

        viewModel.importFile(uri)

        coVerify { mockRelayOverridesRepository.applySettingsPatch(TEXT_INPUT) }
    }

    companion object {
        private const val TEXT_INPUT = "My cool json patch"

        private const val READ_TEXT = "kotlin.io.TextStreamsKt"
    }
}
