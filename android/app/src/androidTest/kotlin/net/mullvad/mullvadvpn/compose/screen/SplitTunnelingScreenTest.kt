package net.mullvad.mullvadvpn.compose.screen

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
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.di.APPS_SCOPE
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.koin.core.context.loadKoinModules
import org.koin.core.context.unloadKoinModules
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope
import org.koin.dsl.module
import org.koin.java.KoinJavaComponent.getKoin

class SplitTunnelingScreenTest {
    @get:Rule val composeTestRule = createComposeRule()
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

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        loadKoinModules(testModule)
        scope = getKoin().getOrCreateScope(APPS_SCOPE, named(APPS_SCOPE))
    }

    @After
    fun tearDown() {
        scope.close()
        unloadKoinModules(testModule)
        unmockkAll()
    }

    @Test
    fun testLoadingState() {
        // Arrange
        composeTestRule.setContent { SplitTunnelingScreen(uiState = SplitTunnelingUiState.Loading) }

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
        composeTestRule.setContent {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState.Data(
                        excludedApps = listOf(excludedApp),
                        includedApps = listOf(includedApp),
                        showSystemApps = false
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
        composeTestRule.setContent {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState.Data(
                        excludedApps = emptyList(),
                        includedApps = listOf(includedApp),
                        showSystemApps = false
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
        composeTestRule.setContent {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState.Data(
                        excludedApps = listOf(excludedApp),
                        includedApps = listOf(includedApp),
                        showSystemApps = false
                    ),
                addToExcluded = mockedClickHandler
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
        composeTestRule.setContent {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState.Data(
                        excludedApps = listOf(excludedApp),
                        includedApps = listOf(includedApp),
                        showSystemApps = false
                    ),
                removeFromExcluded = mockedClickHandler
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
        composeTestRule.setContent {
            SplitTunnelingScreen(
                uiState =
                    SplitTunnelingUiState.Data(
                        excludedApps = listOf(excludedApp),
                        includedApps = listOf(includedApp),
                        showSystemApps = false
                    ),
                onShowSystemAppsClicked = mockedClickHandler
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
        private const val DESCRIPTION =
            "Split tunneling makes it possible to select which applications should not be routed through the VPN tunnel."
        private const val EXCLUDED_APPLICATIONS = "Excluded applications"
        private const val SHOW_SYSTEM_APPS = "Show system apps"
        private const val ALL_APPLICATIONS = "All applications"
    }
}
