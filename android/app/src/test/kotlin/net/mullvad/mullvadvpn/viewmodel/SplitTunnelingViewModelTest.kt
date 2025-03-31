package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import java.util.concurrent.TimeUnit
import kotlin.test.assertEquals
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.screen.SplitTunnelingNavArgs
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
@Timeout(3000L, unit = TimeUnit.MILLISECONDS)
class SplitTunnelingViewModelTest {

    private val mockedApplicationsProvider = mockk<ApplicationsProvider>()
    private val mockedSplitTunnelingRepository = mockk<SplitTunnelingRepository>()
    private lateinit var testSubject: SplitTunnelingViewModel

    private val excludedApps: MutableStateFlow<Set<AppId>> = MutableStateFlow(emptySet())
    private val enabled: MutableStateFlow<Boolean> = MutableStateFlow(true)

    @BeforeEach
    fun setup() {
        every { mockedSplitTunnelingRepository.splitTunnelingEnabled } returns enabled
        every { mockedSplitTunnelingRepository.excludedApps } returns excludedApps
    }

    @AfterEach
    fun tearDown() {
        testSubject.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        initTestSubject(emptyList())
        val actualState: SplitTunnelingUiState = testSubject.uiState.value

        val initialExpectedState = SplitTunnelingUiState.Loading(enabled = false)

        assertEquals(initialExpectedState, actualState)

        verify(exactly = 1) { mockedApplicationsProvider.getAppsList() }
    }

    @Test
    fun `empty app list should work`() = runTest {
        initTestSubject(emptyList())
        val expectedState =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = emptyList(),
                showSystemApps = false,
            )
        testSubject.uiState.test { assertEquals(expectedState, awaitItem()) }
    }

    @Test
    fun `includedApps and excludedApps should both be included in uiState`() = runTest {
        val appExcluded = AppData("test.excluded", 0, "testName1")
        val appNotExcluded = AppData("test.not.excluded", 0, "testName2")

        initTestSubject(listOf(appExcluded, appNotExcluded))
        excludedApps.value = setOf(AppId(appExcluded.packageName))

        val expectedState =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = listOf(appExcluded),
                includedApps = listOf(appNotExcluded),
                showSystemApps = false,
            )

        testSubject.uiState.test {
            val actualState = awaitItem()
            assertEquals(expectedState, actualState)
        }
    }

    @Test
    fun `include app should work`() = runTest {
        val app = AppData("test", 0, "testName")

        initTestSubject(listOf(app))
        excludedApps.value = setOf(AppId(app.packageName))

        val expectedStateBeforeAction =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = listOf(app),
                includedApps = emptyList(),
                showSystemApps = false,
            )
        val expectedStateAfterAction =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = listOf(app),
                showSystemApps = false,
            )
        coEvery { mockedSplitTunnelingRepository.includeApp(AppId(app.packageName)) } returns
            Unit.right()

        testSubject.uiState.test {
            assertEquals(expectedStateBeforeAction, awaitItem())
            testSubject.onIncludeAppClick(app.packageName)
            excludedApps.value = emptySet()
            assertEquals(expectedStateAfterAction, awaitItem())

            coVerify { mockedSplitTunnelingRepository.includeApp(AppId(app.packageName)) }
        }
    }

    @Test
    fun `onExcludeApp should result in new uiState with app excluded`() = runTest {
        val app = AppData("test", 0, "testName")

        initTestSubject(listOf(app))

        val expectedStateBeforeAction =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = listOf(app),
                showSystemApps = false,
            )

        val expectedStateAfterAction =
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps = listOf(app),
                includedApps = emptyList(),
                showSystemApps = false,
            )

        coEvery { mockedSplitTunnelingRepository.excludeApp(AppId(app.packageName)) } returns
            Unit.right()

        testSubject.uiState.test {
            assertEquals(expectedStateBeforeAction, awaitItem())
            testSubject.onExcludeAppClick(app.packageName)
            excludedApps.value = setOf(AppId(app.packageName))
            assertEquals(expectedStateAfterAction, awaitItem())

            coVerify { mockedSplitTunnelingRepository.excludeApp(AppId(app.packageName)) }
        }
    }

    @Test
    fun `when split tunneling is disabled uiState should be disabled`() = runTest {
        initTestSubject(emptyList())
        enabled.value = false

        val expectedState = SplitTunnelingUiState.ShowAppList(enabled = false)

        testSubject.uiState.test {
            val actualState = awaitItem()
            assertEquals(expectedState, actualState)
        }
    }

    private fun initTestSubject(appList: List<AppData>) {
        every { mockedApplicationsProvider.getAppsList() } returns appList
        testSubject =
            SplitTunnelingViewModel(
                mockedApplicationsProvider,
                mockedSplitTunnelingRepository,
                savedStateHandle = SplitTunnelingNavArgs().toSavedStateHandle(),
                UnconfinedTestDispatcher(),
            )
    }
}
