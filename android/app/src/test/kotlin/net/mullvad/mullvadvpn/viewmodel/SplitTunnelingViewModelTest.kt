package net.mullvad.mullvadvpn.viewmodel

import androidx.annotation.StringRes
import androidx.lifecycle.viewModelScope
import io.mockk.Runs
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import io.mockk.verifyAll
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.assertLists
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.Timeout

class SplitTunnelingViewModelTest {
    @get:Rule
    val testCoroutineRule = TestCoroutineRule()

    @get:Rule
    val timeout = Timeout(3000L, TimeUnit.MILLISECONDS)
    private val mockedApplicationsProvider = mockk<ApplicationsProvider>()
    private val mockedSplitTunneling = mockk<SplitTunneling>()
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
    fun test_has_progress_on_start() = runBlockingTest(testCoroutineRule.testDispatcher) {
        initTestSubject(emptyList())
        val actualList: List<ListItemData> = testSubject.listItems.first()

        val initialExpectedList = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(0),
            createProgressItem()
        )

        assertLists(initialExpectedList, actualList)

        verify(exactly = 1) {
            mockedApplicationsProvider.getAppsList()
        }
    }

    @Test
    fun test_empty_app_list() = runBlockingTest(testCoroutineRule.testDispatcher) {
        initTestSubject(emptyList())
        testSubject.processIntent(ViewIntent.ViewIsReady)
        val actualList = testSubject.listItems.first()
        val expectedList = listOf(createTextItem(R.string.split_tunneling_description))
        assertLists(expectedList, actualList)
    }

    @Test
    fun test_apps_list_delivered() = runBlockingTest(testCoroutineRule.testDispatcher) {
        val appExcluded = AppData("test.excluded", 0, "testName1")
        val appNotExcluded = AppData("test.not.excluded", 0, "testName2")
        every { mockedSplitTunneling.isAppExcluded(appExcluded.packageName) } returns true
        every { mockedSplitTunneling.isAppExcluded(appNotExcluded.packageName) } returns false

        initTestSubject(listOf(appExcluded, appNotExcluded))
        testSubject.processIntent(ViewIntent.ViewIsReady)

        val actualList = testSubject.listItems.first()
        val expectedList = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(0),
            createMainItem(R.string.exclude_applications),
            createApplicationItem(appExcluded, true),
            createDivider(1),
            createSwitchItem(R.string.show_system_apps, false),
            createMainItem(R.string.all_applications),
            createApplicationItem(appNotExcluded, false),
        )

        assertLists(expectedList, actualList)
        verifyAll {
            mockedSplitTunneling.enabled
            mockedSplitTunneling.isAppExcluded(appExcluded.packageName)
            mockedSplitTunneling.isAppExcluded(appNotExcluded.packageName)
        }
    }

    @Test
    fun test_remove_app_from_excluded() = runBlockingTest(testCoroutineRule.testDispatcher) {
        val app = AppData("test", 0, "testName")
        every { mockedSplitTunneling.isAppExcluded(app.packageName) } returns true
        every { mockedSplitTunneling.includeApp(app.packageName) } just Runs

        initTestSubject(listOf(app))
        testSubject.processIntent(ViewIntent.ViewIsReady)

        val listBeforeAction = testSubject.listItems.first()
        val expectedListBeforeAction = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(0),
            createMainItem(R.string.exclude_applications),
            createApplicationItem(app, true),
        )

        assertLists(expectedListBeforeAction, listBeforeAction)

        val item = listBeforeAction.first { it.identifier == app.packageName }
        testSubject.processIntent(ViewIntent.ChangeApplicationGroup(item))

        val itemsAfterAction = testSubject.listItems.first()
        val expectedList = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(1),
            createSwitchItem(R.string.show_system_apps, false),
            createMainItem(R.string.all_applications),
            createApplicationItem(app, false),
        )

        assertLists(expectedList, itemsAfterAction)

        verifyAll {
            mockedSplitTunneling.enabled
            mockedSplitTunneling.isAppExcluded(app.packageName)
            mockedSplitTunneling.includeApp(app.packageName)
        }
    }

    @Test
    fun test_add_app_to_excluded() = runBlockingTest(testCoroutineRule.testDispatcher) {
        val app = AppData("test", 0, "testName")
        every { mockedSplitTunneling.isAppExcluded(app.packageName) } returns false
        every { mockedSplitTunneling.excludeApp(app.packageName) } just Runs
        initTestSubject(listOf(app))
        testSubject.processIntent(ViewIntent.ViewIsReady)

        val listBeforeAction = testSubject.listItems.first()
        val expectedListBeforeAction = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(1),
            createSwitchItem(R.string.show_system_apps, false),
            createMainItem(R.string.all_applications),
            createApplicationItem(app, false),
        )

        assertLists(expectedListBeforeAction, listBeforeAction)

        val item = listBeforeAction.first { it.identifier == app.packageName }
        testSubject.processIntent(ViewIntent.ChangeApplicationGroup(item))

        val itemsAfterAction = testSubject.listItems.first()
        val expectedList = listOf(
            createTextItem(R.string.split_tunneling_description),
            createDivider(0),
            createMainItem(R.string.exclude_applications),
            createApplicationItem(app, true),
        )

        assertLists(expectedList, itemsAfterAction)

        verifyAll {
            mockedSplitTunneling.enabled
            mockedSplitTunneling.isAppExcluded(app.packageName)
            mockedSplitTunneling.excludeApp(app.packageName)
        }
    }

    private fun initTestSubject(appList: List<AppData>) {
        every { mockedApplicationsProvider.getAppsList() } returns appList
        testSubject = SplitTunnelingViewModel(
            mockedApplicationsProvider,
            mockedSplitTunneling,
            testCoroutineRule.testDispatcher
        )
    }

    private fun createApplicationItem(
        appData: AppData,
        checked: Boolean
    ): ListItemData = ListItemData.build(appData.packageName) {
        type = ListItemData.APPLICATION
        text = appData.name
        iconRes = appData.iconRes
        action = ListItemData.ItemAction(appData.packageName)
        widget = WidgetState.ImageState(
            if (checked) R.drawable.ic_icons_remove else R.drawable.ic_icons_add
        )
    }

    private fun createDivider(id: Int): ListItemData = ListItemData.build("space_$id") {
        type = ListItemData.DIVIDER
    }

    private fun createMainItem(@StringRes text: Int): ListItemData =
        ListItemData.build("header_$text") {
            type = ListItemData.ACTION
            textRes = text
        }

    private fun createTextItem(@StringRes text: Int): ListItemData =
        ListItemData.build("text_$text") {
            type = ListItemData.PLAIN
            textRes = text
            action = ListItemData.ItemAction(text.toString())
        }

    private fun createProgressItem(): ListItemData = ListItemData.build(identifier = "progress") {
        type = ListItemData.PROGRESS
    }

    private fun createSwitchItem(@StringRes text: Int, checked: Boolean): ListItemData =
        ListItemData.build(identifier = "switch_$text") {
            type = ListItemData.ACTION
            textRes = text
            action = ListItemData.ItemAction(text.toString())
            widget = WidgetState.SwitchState(checked)
        }
}
