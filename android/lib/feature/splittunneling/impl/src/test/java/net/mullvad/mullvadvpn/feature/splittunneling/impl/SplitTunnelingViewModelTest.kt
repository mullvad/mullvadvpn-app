package net.mullvad.mullvadvpn.feature.splittunneling.impl

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import java.util.concurrent.TimeUnit
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.AppData
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.SplitTunnelingUseCase
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
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
    private val mockedUserPreferencesRepository = mockk<UserPreferencesRepository>()
    private lateinit var testSubject: SplitTunnelingViewModel

    private val excludedApps: MutableStateFlow<Set<PackageName>> = MutableStateFlow(emptySet())
    private val enabled: MutableStateFlow<Boolean> = MutableStateFlow(true)
    private val showSystemApps: MutableStateFlow<Boolean> = MutableStateFlow(false)

    @BeforeEach
    fun setup() {
        every { mockedSplitTunnelingRepository.splitTunnelingEnabled } returns enabled
        every { mockedSplitTunnelingRepository.excludedApps } returns excludedApps
        every { mockedUserPreferencesRepository.showSystemAppsSplitTunneling() } returns
            showSystemApps
    }

    @AfterEach
    fun tearDown() {
        testSubject.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        initTestSubject(emptyList())
        val actualState: Lc<Loading, SplitTunnelingUiState> = testSubject.uiState.value

        val initialExpectedState = Lc.Loading(Loading())

        assertIs<Lc.Loading<Loading>>(actualState)
        assertEquals(initialExpectedState, actualState)

        verify(exactly = 1) { mockedApplicationsProvider.apps() }
    }

    @Test
    fun `empty app list should work`() = runTest {
        initTestSubject(emptyList())
        val expectedState =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = emptyList(),
                showSystemApps = false,
            )
        testSubject.uiState.test {
            val item = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(item)
            assertEquals(expectedState, item.value)
        }
    }

    @Test
    fun `includedApps and excludedApps should both be included in uiState`() = runTest {
        val appExcluded = AppData(PackageName("test.excluded"), 0, "testName1")
        val appNotExcluded = AppData(PackageName("test.not.excluded"), 0, "testName2")

        initTestSubject(listOf(appExcluded, appNotExcluded))
        excludedApps.value = setOf(appExcluded.packageName)

        val expectedState =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = listOf(appExcluded),
                includedApps = listOf(appNotExcluded),
                showSystemApps = false,
            )

        testSubject.uiState.test {
            val actualState = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(actualState)
            assertEquals(expectedState, actualState.value)
        }
    }

    @Test
    fun `include app should work`() = runTest {
        val app = AppData(PackageName("test"), 0, "testName")

        initTestSubject(listOf(app))
        excludedApps.value = setOf(app.packageName)

        val expectedStateBeforeAction =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = listOf(app),
                includedApps = emptyList(),
                showSystemApps = false,
            )
        val expectedStateAfterAction =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = listOf(app),
                showSystemApps = false,
            )
        coEvery { mockedSplitTunnelingRepository.includeApp(app.packageName) } returns Unit.right()

        testSubject.uiState.test {
            val beforeAction = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(beforeAction)
            assertEquals(expectedStateBeforeAction, beforeAction.value)
            testSubject.onIncludeAppClick(app.packageName)
            excludedApps.value = emptySet()
            val afterAction = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(afterAction)
            assertEquals(expectedStateAfterAction, afterAction.value)

            coVerify { mockedSplitTunnelingRepository.includeApp(app.packageName) }
        }
    }

    @Test
    fun `onExcludeApp should result in new uiState with app excluded`() = runTest {
        val app = AppData(PackageName("test"), 0, "testName")

        initTestSubject(listOf(app))

        val expectedStateBeforeAction =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = emptyList(),
                includedApps = listOf(app),
                showSystemApps = false,
            )

        val expectedStateAfterAction =
            SplitTunnelingUiState(
                enabled = true,
                excludedApps = listOf(app),
                includedApps = emptyList(),
                showSystemApps = false,
            )

        coEvery { mockedSplitTunnelingRepository.excludeApp(app.packageName) } returns Unit.right()

        testSubject.uiState.test {
            val beforeAction = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(beforeAction)
            assertEquals(expectedStateBeforeAction, beforeAction.value)
            testSubject.onExcludeAppClick(app.packageName)
            excludedApps.value = setOf(app.packageName)
            val afterAction = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(afterAction)
            assertEquals(expectedStateAfterAction, afterAction.value)

            coVerify { mockedSplitTunnelingRepository.excludeApp(app.packageName) }
        }
    }

    @Test
    fun `when split tunneling is disabled uiState should be disabled`() = runTest {
        initTestSubject(emptyList())
        enabled.value = false

        val expectedState = SplitTunnelingUiState(enabled = false)

        testSubject.uiState.test {
            val actualState = awaitItem()
            assertIs<Lc.Content<SplitTunnelingUiState>>(actualState)
            assertEquals(expectedState, actualState.value)
        }
    }

    private fun initTestSubject(appList: List<AppData>) {
        every { mockedApplicationsProvider.apps() } returns appList
        testSubject =
            SplitTunnelingViewModel(
                isModal = false,
                mockedSplitTunnelingRepository,
                mockedUserPreferencesRepository,
                SplitTunnelingUseCase(
                    mockedSplitTunnelingRepository,
                    mockedApplicationsProvider,
                    mockedUserPreferencesRepository,
                ),
                UnconfinedTestDispatcher(),
            )
    }
}
