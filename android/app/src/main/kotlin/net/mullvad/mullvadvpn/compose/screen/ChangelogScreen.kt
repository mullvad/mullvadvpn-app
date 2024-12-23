package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createOpenFullChangeLogHook
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ChangeLogSideEffect
import net.mullvad.mullvadvpn.viewmodel.ChangelogUiState
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.koin.androidx.compose.koinViewModel

data class ChangelogNavArgs(val isModal: Boolean = false)

@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = ChangelogNavArgs::class,
)
@Composable
fun Changelog(navController: NavController) {
    val viewModel = koinViewModel<ChangelogViewModel>()

    val uiState = viewModel.uiState.collectAsStateWithLifecycle()

    val openAccountPage = LocalUriHandler.current.createOpenFullChangeLogHook()
    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is ChangeLogSideEffect.OpenFullChangelog -> openAccountPage()
        }
    }
    LaunchedEffect(Unit) { viewModel.dismissChangelogNotification() }

    ChangelogScreen(
        uiState.value,
        onBackClick = { navController.navigateUp() },
        onSeeFullChangelog = viewModel::onSeeFullChangelog,
    )
}

@Composable
fun ChangelogScreen(
    state: ChangelogUiState,
    onBackClick: () -> Unit,
    onSeeFullChangelog: () -> Unit,
) {

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.changelog_title),
        navigationIcon = {
            if (state.isModal) {
                NavigateCloseIconButton(onBackClick)
            } else {
                NavigateBackIconButton(onNavigateBack = onBackClick)
            }
        },
    ) { modifier ->
        Column(modifier = modifier.padding(horizontal = Dimens.mediumPadding)) {
            Column(
                Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
            ) {
                Text(
                    text = state.version,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )

                if (state.changes.isEmpty()) {
                    Text(
                        text = stringResource(R.string.changelog_empty),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                } else {
                    state.changes.forEach { changeItem -> ChangeListItem(text = changeItem) }
                }
            }
            Box(modifier = Modifier.padding(Dimens.mediumPadding).fillMaxWidth()) {
                PrimaryButton(
                    onClick = onSeeFullChangelog,
                    text = stringResource(R.string.see_full_changelog),
                    trailingIcon = {
                        Icon(
                            imageVector = Icons.AutoMirrored.Default.OpenInNew,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSurface,
                        )
                    },
                )
            }
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
            onSeeFullChangelog = {},
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
            onSeeFullChangelog = {},
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
            onSeeFullChangelog = {},
        )
    }
}
