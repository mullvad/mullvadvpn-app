package net.mullvad.mullvadvpn.compose.screen

import android.graphics.Bitmap
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.AppListState
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.koin.core.context.loadKoinModules
import org.koin.core.context.unloadKoinModules
import org.koin.dsl.module

class SplitTunnelingScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    private val mockBitmap: Bitmap = Bitmap.createBitmap(10, 10, Bitmap.Config.ARGB_8888)
    private val testModule = module {
        single {
            mockk<ApplicationsIconManager>().apply {
                every { getAppIcon(any()) } returns mockBitmap
            }
        }
    }

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        loadKoinModules(testModule)
    }

    @After
    fun tearDown() {
        unloadKoinModules(testModule)
        unmockkAll()
    }

    @Test
    fun testLoadingState() {
        // Arrange
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(uiState = SplitTunnelingUiState())
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(TITLE).assertExists()
            onNodeWithText(DESCRIPTION).assertExists()
            onNodeWithText(EXCLUDED_APPLICATIONS).assertDoesNotExist()
            onNodeWithText(SHOW_SYSTEM_APPS).assertDoesNotExist()
            onNodeWithText(ALL_APPLICATIONS).assertDoesNotExist()
        }
    }

    @Test
    fun testListDisplayed() {
        // Arrange
        val excludedApp =
            AppData(packageName = EXCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = EXCLUDED_APP_NAME)
        val includedApp =
            AppData(packageName = INCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = INCLUDED_APP_NAME)
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState(
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = listOf(excludedApp),
                                includedApps = listOf(includedApp),
                                showSystemApps = false
                            )
                    )
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(TITLE).assertExists()
            onNodeWithText(DESCRIPTION).assertExists()
            onNodeWithText(EXCLUDED_APPLICATIONS).assertExists()
            onNodeWithText(EXCLUDED_APP_NAME).assertExists()
            onNodeWithText(SHOW_SYSTEM_APPS).assertExists()
            onNodeWithText(ALL_APPLICATIONS).assertExists()
            onNodeWithText(INCLUDED_APP_NAME).assertExists()
        }
    }

    @Test
    fun testNoExcludedApps() {
        // Arrange
        val includedApp =
            AppData(packageName = INCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = INCLUDED_APP_NAME)
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState(
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = emptyList(),
                                includedApps = listOf(includedApp),
                                showSystemApps = false
                            )
                    )
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(TITLE).assertExists()
            onNodeWithText(DESCRIPTION).assertExists()
            onNodeWithText(EXCLUDED_APPLICATIONS).assertDoesNotExist()
            onNodeWithText(EXCLUDED_APP_NAME).assertDoesNotExist()
            onNodeWithText(SHOW_SYSTEM_APPS).assertExists()
            onNodeWithText(ALL_APPLICATIONS).assertExists()
            onNodeWithText(INCLUDED_APP_NAME).assertExists()
        }
    }

    @Test
    fun testClickIncludedItem() {
        // Arrange
        val excludedApp =
            AppData(packageName = EXCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = EXCLUDED_APP_NAME)
        val includedApp =
            AppData(packageName = INCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = INCLUDED_APP_NAME)
        val mockedClickHandler: (String) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState(
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = listOf(excludedApp),
                                includedApps = listOf(includedApp),
                                showSystemApps = false
                            )
                    ),
                onExcludeAppClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText(INCLUDED_APP_NAME).performClick()

        // Assert
        verify { mockedClickHandler.invoke(INCLUDED_APP_PACKAGE_NAME) }
    }

    @Test
    fun testClickExcludedItem() {
        // Arrange
        val excludedApp =
            AppData(packageName = EXCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = EXCLUDED_APP_NAME)
        val includedApp =
            AppData(packageName = INCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = INCLUDED_APP_NAME)
        val mockedClickHandler: (String) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState(
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = listOf(excludedApp),
                                includedApps = listOf(includedApp),
                                showSystemApps = false
                            )
                    ),
                onIncludeAppClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText(EXCLUDED_APP_NAME).performClick()

        // Assert
        verify { mockedClickHandler.invoke(EXCLUDED_APP_PACKAGE_NAME) }
    }

    @Test
    fun testClickShowSystemApps() {
        // Arrange
        val excludedApp =
            AppData(packageName = EXCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = EXCLUDED_APP_NAME)
        val includedApp =
            AppData(packageName = INCLUDED_APP_PACKAGE_NAME, iconRes = 0, name = INCLUDED_APP_NAME)
        val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState(
                        appListState =
                            AppListState.ShowAppList(
                                excludedApps = listOf(excludedApp),
                                includedApps = listOf(includedApp),
                                showSystemApps = false
                            )
                    ),
                onShowSystemAppsClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText(SHOW_SYSTEM_APPS).performClick()

        // Assert
        verify { mockedClickHandler.invoke(true) }
    }

    companion object {
        private const val EXCLUDED_APP_PACKAGE_NAME = "excluded-pkg"
        private const val EXCLUDED_APP_NAME = "Excluded Name"
        private const val INCLUDED_APP_PACKAGE_NAME = "included-pkg"
        private const val INCLUDED_APP_NAME = "Included Name"
        private const val TITLE = "Split tunneling"
        private const val DESCRIPTION = "Choose the apps you want to exclude from the VPN tunnel."
        private const val EXCLUDED_APPLICATIONS = "Excluded applications"
        private const val SHOW_SYSTEM_APPS = "Show system apps"
        private const val ALL_APPLICATIONS = "All applications"
    }
}
