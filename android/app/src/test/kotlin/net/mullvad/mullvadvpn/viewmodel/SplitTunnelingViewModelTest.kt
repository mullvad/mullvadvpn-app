package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.invoke
import io.mockk.just
import io.mockk.mockk
import io.mockk.runs
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import io.mockk.verifyAll
import java.util.concurrent.TimeUnit
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.AppListState
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.Timeout

class SplitTunnelingViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @get:Rule val timeout = Timeout(3000L, TimeUnit.MILLISECONDS)
    private val mockedApplicationsProvider = mockk<ApplicationsProvider>()
    private val mockedSplitTunneling = mockk<SplitTunneling>()
    private val mockedServiceConnectionManager = mockk<ServiceConnectionManager>()
    private val mockedServiceConnectionContainer = mockk<ServiceConnectionContainer>()
    private lateinit var testSubject: SplitTunnelingViewModel

    @Before
    fun setup() {
        every { mockedSplitTunneling.enabled } returns true
    }

    @After
    fun tearDown() {
        testSubject.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_has_progress_on_start() =
        runTest(testCoroutineRule.testDispatcher) {
            initTestSubject(emptyList())
            val actualState: SplitTunnelingUiState = testSubject.uiState.value

            val initialExpectedState = SplitTunnelingUiState()

            assertEquals(initialExpectedState, actualState)

            verify(exactly = 1) { mockedApplicationsProvider.getAppsList() }
        }

    @Test
    fun test_empty_app_list() =
        runTest(testCoroutineRule.testDispatcher) {
            every { mockedSplitTunneling.excludedAppsChange = captureLambda() } answers
                {
                    lambda<(Set<String>) -> Unit>().invoke(emptySet())
                }
            every { mockedSplitTunneling.enabledChange = captureLambda() } answers
                {
                    lambda<(Boolean) -> Unit>().invoke(true)
                }

            initTestSubject(emptyList())
            val expectedState =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = emptyList(),
                            includedApps = emptyList(),
                            showSystemApps = false
                        )
                )
            testSubject.uiState.test { assertEquals(expectedState, awaitItem()) }
        }

    @Test
    fun test_apps_list_delivered() =
        runTest(testCoroutineRule.testDispatcher) {
            val appExcluded = AppData("test.excluded", 0, "testName1")
            val appNotExcluded = AppData("test.not.excluded", 0, "testName2")
            every { mockedSplitTunneling.excludedAppsChange = captureLambda() } answers
                {
                    lambda<(Set<String>) -> Unit>().invoke(setOf(appExcluded.packageName))
                }
            every { mockedSplitTunneling.enabledChange = captureLambda() } answers
                {
                    lambda<(Boolean) -> Unit>().invoke(true)
                }

            every { mockedSplitTunneling.enableSplitTunneling(any()) } just runs

            initTestSubject(listOf(appExcluded, appNotExcluded))

            val expectedState =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = listOf(appExcluded),
                            includedApps = listOf(appNotExcluded),
                            showSystemApps = false
                        )
                )

            testSubject.uiState.test {
                val actualState = awaitItem()
                assertEquals(expectedState, actualState)
                verifyAll {
                    mockedSplitTunneling.enabledChange = any()
                    mockedSplitTunneling.excludedAppsChange = any()
                }
            }
        }

    @Test
    fun test_include_app() =
        runTest(testCoroutineRule.testDispatcher) {
            var excludedAppsCallback = slot<(Set<String>) -> Unit>()
            val app = AppData("test", 0, "testName")
            every { mockedSplitTunneling.includeApp(app.packageName) } just runs
            every { mockedSplitTunneling.excludedAppsChange = captureLambda() } answers
                {
                    excludedAppsCallback = lambda()
                    excludedAppsCallback.invoke(setOf(app.packageName))
                }
            every { mockedSplitTunneling.enabledChange = captureLambda() } answers
                {
                    lambda<(Boolean) -> Unit>().invoke(true)
                }

            initTestSubject(listOf(app))

            val expectedStateBeforeAction =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = listOf(app),
                            includedApps = emptyList(),
                            showSystemApps = false
                        )
                )
            val expectedStateAfterAction =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = emptyList(),
                            includedApps = listOf(app),
                            showSystemApps = false
                        )
                )

            testSubject.uiState.test {
                assertEquals(expectedStateBeforeAction, awaitItem())
                testSubject.onIncludeAppClick(app.packageName)
                excludedAppsCallback.invoke(emptySet())
                assertEquals(expectedStateAfterAction, awaitItem())

                verifyAll {
                    mockedSplitTunneling.enabledChange = any()
                    mockedSplitTunneling.excludedAppsChange = any()
                    mockedSplitTunneling.includeApp(app.packageName)
                }
            }
        }

    @Test
    fun test_add_app_to_excluded() =
        runTest(testCoroutineRule.testDispatcher) {
            var excludedAppsCallback = slot<(Set<String>) -> Unit>()
            val app = AppData("test", 0, "testName")
            every { mockedSplitTunneling.excludeApp(app.packageName) } just runs
            every { mockedSplitTunneling.excludedAppsChange = captureLambda() } answers
                {
                    excludedAppsCallback = lambda()
                    excludedAppsCallback.invoke(emptySet())
                }
            every { mockedSplitTunneling.enabledChange = captureLambda() } answers
                {
                    lambda<(Boolean) -> Unit>().invoke(true)
                }

            initTestSubject(listOf(app))

            val expectedStateBeforeAction =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = emptyList(),
                            includedApps = listOf(app),
                            showSystemApps = false
                        )
                )

            val expectedStateAfterAction =
                SplitTunnelingUiState(
                    enabled = true,
                    appListState =
                        AppListState.ShowAppList(
                            excludedApps = listOf(app),
                            includedApps = emptyList(),
                            showSystemApps = false
                        )
                )

            testSubject.uiState.test {
                assertEquals(expectedStateBeforeAction, awaitItem())
                testSubject.onExcludeAppClick(app.packageName)
                excludedAppsCallback.invoke(setOf(app.packageName))
                assertEquals(expectedStateAfterAction, awaitItem())

                verifyAll {
                    mockedSplitTunneling.enabledChange = any()
                    mockedSplitTunneling.excludedAppsChange = any()
                    mockedSplitTunneling.excludeApp(app.packageName)
                }
            }
        }

    @Test
    fun test_disabled_state() =
        runTest(testCoroutineRule.testDispatcher) {
            every { mockedSplitTunneling.excludedAppsChange = captureLambda() } answers
                {
                    lambda<(Set<String>) -> Unit>().invoke(emptySet())
                }
            every { mockedSplitTunneling.enabledChange = captureLambda() } answers
                {
                    lambda<(Boolean) -> Unit>().invoke(false)
                }

            initTestSubject(emptyList())

            val expectedState =
                SplitTunnelingUiState(enabled = false, appListState = AppListState.Disabled)

            testSubject.uiState.test {
                val actualState = awaitItem()
                assertEquals(expectedState, actualState)
            }
        }

    private fun initTestSubject(appList: List<AppData>) {
        every { mockedApplicationsProvider.getAppsList() } returns appList
        every { mockedServiceConnectionManager.connectionState } returns
            MutableStateFlow(
                ServiceConnectionState.ConnectedReady(mockedServiceConnectionContainer)
            )
        every { mockedServiceConnectionContainer.splitTunneling } returns mockedSplitTunneling
        testSubject =
            SplitTunnelingViewModel(
                mockedApplicationsProvider,
                mockedServiceConnectionManager,
                testCoroutineRule.testDispatcher
            )
    }
}
