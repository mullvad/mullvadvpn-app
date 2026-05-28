package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import androidx.compose.material3.SnackbarHostState
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class MultihopMigrationScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: MultihopMigrationUiState,
        onCloseClick: () -> Unit = {},
        onBackClick: () -> Unit = {},
        onNextClick: () -> Unit = {},
        onFinishMigration: () -> Unit = {},
    ) {
        setContentWithTheme {
            MultihopMigrationScreen(
                state = state,
                snackbarHostState = SnackbarHostState(),
                onCloseClick = onCloseClick,
                onBackClick = onBackClick,
                onNextClick = onNextClick,
                onFinishMigration = onFinishMigration,
                onSetEntry = {},
                onSetMultihopMode = {},
            )
        }
    }

    @Test
    fun testNewMultihopModeOffToNever() = composeExtension.use {
        // Arrange
        val pages =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER))
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("New multihop modes").assertExists()
        onNodeWithText("Your multihop setting was migrated from “Off” to “Never”", substring = true)
            .assertExists()
    }

    @Test
    fun testNewMultihopModeOffToWhenNeeded() = composeExtension.use {
        // Arrange
        val pages =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_WHEN_NEEDED))
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("New multihop modes").assertExists()
        onNodeWithText(
                "Your multihop setting was migrated from “Off” to “When needed”",
                substring = true,
            )
            .assertExists()
    }

    @Test
    fun testNewMultihopModeOffToAlways() = composeExtension.use {
        // Arrange
        val pages =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_ALWAYS))
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("New multihop modes").assertExists()
        onNodeWithText(
                "Your multihop setting was migrated from “Off” to “Always”",
                substring = true,
            )
            .assertExists()
    }

    @Test
    fun testNewMultihopModeOnToAlways() = composeExtension.use {
        // Arrange
        val pages =
            listOf(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("New multihop modes").assertExists()
        onNodeWithText("Your multihop setting was migrated from “On” to Always”", substring = true)
            .assertExists()
    }

    @Test
    fun testDirectOnlyRemovedPage() = composeExtension.use {
        // Arrange
        val pages = listOf(MultihopMigrationPage.DirectOnlyRemoved)
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("“Direct Only” removed").assertExists()
    }

    @Test
    fun testSeparateFiltersPage() = composeExtension.use {
        // Arrange
        val pages = listOf(MultihopMigrationPage.SeparateFilters)
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("Separate filters").assertExists()
    }

    @Test
    fun testSuggestedMultihopEntryPage() = composeExtension.use {
        // Arrange
        val pages = listOf(MultihopMigrationPage.SuggestedMultihopEntry)
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("Suggested multihop entry").assertExists()
    }

    @Test
    fun testSuggestedActionPage() = composeExtension.use {
        // Arrange
        val pages = listOf(MultihopMigrationPage.SuggestedAction)
        initScreen(state = MultihopMigrationUiState(pages, currentPageIndex = 0))

        // Assert
        onNodeWithText("Suggested action").assertExists()
    }

    @Test
    fun testClickBack_callsOnBackClick() = composeExtension.use {
        // Arrange
        val onBackClick: () -> Unit = mockk(relaxed = true)
        val pages =
            listOf(
                MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                MultihopMigrationPage.SeparateFilters,
                MultihopMigrationPage.SuggestedAction,
            )
        initScreen(
            state = MultihopMigrationUiState(pages, currentPageIndex = 1),
            onBackClick = onBackClick,
        )

        // Act
        onNodeWithText("Back").performClick()

        // Assert
        verify(exactly = 1) { onBackClick.invoke() }
    }

    @Test
    fun testClickNext_callsOnNextClick() = composeExtension.use {
        // Arrange
        val onNextClick: () -> Unit = mockk(relaxed = true)
        val pages =
            listOf(
                MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER),
                MultihopMigrationPage.SeparateFilters,
                MultihopMigrationPage.SuggestedAction,
            )
        initScreen(
            state = MultihopMigrationUiState(pages, currentPageIndex = 0),
            onNextClick = onNextClick,
        )

        // Act
        onNodeWithText("Next").performClick()

        // Assert
        verify(exactly = 1) { onNextClick.invoke() }
    }
}
