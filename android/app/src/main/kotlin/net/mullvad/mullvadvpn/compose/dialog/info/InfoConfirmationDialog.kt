package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar

@Preview
@Composable
private fun PreviewInfoConfirmationDialog() {
    AppTheme {
        InfoConfirmationDialog(
            navigator = EmptyResultBackNavigator(),
            titleType = InfoConfirmationDialogTitleType.IconAndTitle("Informative title"),
            confirmButtonTitle = stringResource(R.string.enable_anyway),
            cancelButtonTitle = stringResource(R.string.back),
        ) {
            Text(
                text = "Info text paragraph one.",
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.fillMaxWidth(),
            )

            Spacer(modifier = Modifier.height(Dimens.verticalSpace))

            Text(
                text = "More text here.",
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

sealed interface InfoConfirmationDialogTitleType {
    data object IconOnly : InfoConfirmationDialogTitleType

    data class TitleOnly(val title: String) : InfoConfirmationDialogTitleType

    data class IconAndTitle(val title: String) : InfoConfirmationDialogTitleType
}

@Composable
fun InfoConfirmationDialog(
    navigator: ResultBackNavigator<Boolean>,
    titleType: InfoConfirmationDialogTitleType,
    confirmButtonTitle: String,
    cancelButtonTitle: String,
    content: @Composable (() -> Unit)? = null,
) {
    val title =
        when (titleType) {
            is InfoConfirmationDialogTitleType.TitleOnly -> titleType.title
            is InfoConfirmationDialogTitleType.IconAndTitle -> titleType.title
            InfoConfirmationDialogTitleType.IconOnly -> null
        }

    val showIcon =
        when (titleType) {
            is InfoConfirmationDialogTitleType.IconOnly,
            is InfoConfirmationDialogTitleType.IconAndTitle -> true
            is InfoConfirmationDialogTitleType.TitleOnly -> false
        }

    AlertDialog(
        onDismissRequest = { navigator.navigateBack(false) },
        title =
            if (title != null) {
                @Composable { Text(title) }
            } else {
                null
            },
        icon =
            if (showIcon) {
                @Composable {
                    Icon(
                        modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                        imageVector = Icons.Default.Error,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
            } else {
                null
            },
        text =
            if (content != null) {
                @Composable {
                    val scrollState = rememberScrollState()
                    Column(
                        Modifier.drawVerticalScrollbar(
                                scrollState,
                                MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar),
                            )
                            .verticalScroll(scrollState),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        content()
                    }
                }
            } else {
                null
            },
        confirmButton = {
            Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing)) {
                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = confirmButtonTitle,
                    onClick = { navigator.navigateBack(true) },
                )

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = cancelButtonTitle,
                    onClick = { navigator.navigateBack(false) },
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
    )
}
