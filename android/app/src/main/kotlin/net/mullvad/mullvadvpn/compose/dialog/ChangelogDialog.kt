package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.navigation.NavController
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.Changelog
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.koin.androidx.compose.koinViewModel

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun Changelog(navController: NavController, changeLog: Changelog) {
    val viewModel = koinViewModel<ChangelogViewModel>()

    ChangelogDialog(
        changeLog,
        onDismiss = {
            viewModel.markChangelogAsRead()
            navController.navigateUp()
        }
    )
}

@Composable
fun ChangelogDialog(changeLog: Changelog, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = changeLog.version,
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        },
        text = {
            val scrollState: ScrollState = rememberScrollState()
            Column(
                modifier = Modifier.fillMaxWidth().verticalScroll(scrollState),
                verticalArrangement = Arrangement.spacedBy(Dimens.smallPadding),
            ) {
                Text(
                    text = stringResource(R.string.changes_dialog_subtitle),
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.fillMaxWidth()
                )

                changeLog.changes.forEach { changeItem -> ChangeListItem(text = changeItem) }
            }
        },
        confirmButton = {
            PrimaryButton(text = stringResource(R.string.got_it), onClick = onDismiss)
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onBackground
    )
}

@Composable
private fun ChangeListItem(text: String) {
    Column {
        Row {
            Text(
                text = "•",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.width(Dimens.buttonSpacing),
                textAlign = TextAlign.Center
            )
            Text(
                text = text,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onBackground
            )
        }
    }
}

@Preview
@Composable
private fun PreviewChangelogDialogWithSingleShortItem() {
    AppTheme {
        ChangelogDialog(Changelog(changes = listOf("Item 1"), version = "1111.1"), onDismiss = {})
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
        ChangelogDialog(
            Changelog(changes = listOf(longPreviewText, longPreviewText), version = "1111.1"),
            onDismiss = {}
        )
    }
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTenShortItems() {
    AppTheme {
        ChangelogDialog(
            Changelog(
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
                        "Item 10"
                    ),
                version = "1111.1"
            ),
            onDismiss = {}
        )
    }
}
