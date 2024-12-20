package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ChangelogUiState
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.koin.androidx.compose.koinViewModel

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Changelog(navController: NavController) {
    val viewModel = koinViewModel<ChangelogViewModel>()
    val uiState = viewModel.uiState.collectAsStateWithLifecycle()

    ChangelogScreen(uiState.value, onBackClick = { navController.navigateUp() })

    viewModel.dismissChangelogNotification()
}

@Composable
fun ChangelogScreen(state: ChangelogUiState, onBackClick: () -> Unit) {

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.changelog_title),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        Column(
            modifier = modifier.padding(horizontal = Dimens.mediumPadding),
            verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
        ) {
            Text(
                text = state.version,
                style = MaterialTheme.typography.headlineMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )

            state.changes.forEach { changeItem -> ChangeListItem(text = changeItem) }
        }
    }
}

@Composable
private fun ChangeListItem(text: String) {
    Column {
        Row {
            Text(
                text = "•",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.width(Dimens.buttonSpacing),
                textAlign = TextAlign.Center,
            )
            Text(
                text = text,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Preview
@Composable
private fun PreviewChangelogDialogWithSingleShortItem() {
    AppTheme {
        ChangelogScreen(
            ChangelogUiState(changes = listOf("Item 1"), version = "1111.1"),
            onBackClick = {},
        )
    }
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTwoLongItems() {
    val longPreviewText =
        "This is a sample changelog item of a Compose Preview visualization. " +
            "The purpose of this specific sample text is to visualize a long text that will result " +
            "in multiple lines in the changelog dialog."

    AppTheme {
        ChangelogScreen(
            ChangelogUiState(
                changes = listOf(longPreviewText, longPreviewText),
                version = "1111.1",
            ),
            onBackClick = {},
        )
    }
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTenShortItems() {
    AppTheme {
        ChangelogScreen(
            ChangelogUiState(
                changes =
                    listOf(
                        "Item 1",
                        "Item 2",
                        "Item 3",
                        "Item 4",
                        "Item 5",
                        "Item 6",
                        "Item 7",
                        "Item 8",
                        "Item 9",
                        "Item 10",
                    ),
                version = "1111.1",
            ),
            onBackClick = {},
        )
    }
}
