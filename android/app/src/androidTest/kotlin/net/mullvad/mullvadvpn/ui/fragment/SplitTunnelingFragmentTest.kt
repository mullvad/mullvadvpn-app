package net.mullvad.mullvadvpn.ui.fragment

import androidx.fragment.app.testing.launchFragmentInContainer
import androidx.lifecycle.Lifecycle
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.withContentDescription
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.filters.LargeTest
import androidx.test.runner.AndroidJUnit4
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerifyAll
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkClass
import io.mockk.unmockkAll
import io.mockk.verifyAll
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.RecyclerViewMatcher.Companion.withRecyclerView
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.di.APPS_SCOPE
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.koin.core.context.loadKoinModules
import org.koin.core.context.unloadKoinModules
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope
import org.koin.dsl.module
import org.koin.test.KoinTest
import org.koin.test.mock.MockProviderRule
import org.koin.test.mock.declareMock

@RunWith(AndroidJUnit4::class)
@LargeTest
class SplitTunnelingFragmentTest : KoinTest {

    private val mockedViewModel = mockk<SplitTunnelingViewModel>(relaxUnitFun = true)
    private val sharedFlow = MutableSharedFlow<List<ListItemData>>()
    private lateinit var scope: Scope

    private val testModule = module {
        scope(named(APPS_SCOPE)) {
            scoped {
                mockk<ApplicationsIconManager>().apply {
                    every { getAppIcon(any()) } returns mockk(relaxed = true)
                }
            }
        }
    }

    @get:Rule
    val mockProvider = MockProviderRule.create { clazz ->
        when (clazz) {
            SplitTunnelingViewModel::class -> mockedViewModel
            else -> mockkClass(clazz)
        }
    }

    @Before
    fun setUp() {
        loadKoinModules(testModule)
        scope = getKoin().getOrCreateScope(APPS_SCOPE, named(APPS_SCOPE))
        scope.declareMock<SplitTunnelingViewModel>()
        every { mockedViewModel.listItems } returns sharedFlow
        coEvery { mockedViewModel.processIntent(ViewIntent.ViewIsReady) } just Runs
    }

    @After
    fun tearDown() {
        scope.close()
        unloadKoinModules(testModule)
        unmockkAll()
    }

    @Test
    fun test_fragment_title() {
        launchFragmentInContainer<SplitTunnelingFragment>(themeResId = R.style.AppTheme)

        onView(withId(R.id.collapsing_toolbar))
            .check(matches(withContentDescription("Split tunneling")))
    }

    @Test
    fun test_fragment_loading() {
        val scenario = launchFragmentInContainer<SplitTunnelingFragment>(
            themeResId = R.style.AppTheme, initialState = Lifecycle.State.CREATED
        )
        scenario.moveToState(Lifecycle.State.RESUMED)
        sharedFlow.tryEmit(emptyList())

        verifyAll {
            mockedViewModel.listItems
        }
    }

    @Test
    fun test_fragment_list_displayed() = runBlocking {
        launchFragmentInContainer<SplitTunnelingFragment>(
            themeResId = R.style.AppTheme, initialState = Lifecycle.State.RESUMED
        )

        sharedFlow.emit(
            listOf(
                ListItemData.build("testItem") {
                    type = ListItemData.PLAIN
                    text = "Test Item"
                    action = ListItemData.ItemAction(text.toString())
                }
            )
        )

        onView(withRecyclerView(R.id.recyclerView).atPositionOnView(0, R.id.plain_text))
            .check(matches(withText("Test Item")))

        verifyAll {
            mockedViewModel.listItems
        }
    }

    @Test
    fun test_fragment_list_click_application_item() = runBlocking {
        val testListItem = ListItemData.build("test.package.name") {
            type = ListItemData.APPLICATION
            text = "Test App Name"
            action = ListItemData.ItemAction("test.package.name")
            widget = WidgetState.ImageState(R.drawable.ic_icons_add)
        }

        coEvery {
            mockedViewModel.processIntent(ViewIntent.ChangeApplicationGroup(testListItem))
        } just Runs

        launchFragmentInContainer<SplitTunnelingFragment>(
            themeResId = R.style.AppTheme, initialState = Lifecycle.State.RESUMED
        )

        sharedFlow.emit(listOf(testListItem))

        onView(withRecyclerView(R.id.recyclerView).atPositionOnView(0, R.id.itemText))
            .check(matches(withText("Test App Name")))

        onView(withRecyclerView(R.id.recyclerView).atPositionOnView(0)).perform(click())

        coVerifyAll {
            mockedViewModel.listItems
            mockedViewModel.processIntent(ViewIntent.ChangeApplicationGroup(testListItem))
        }
    }
}
