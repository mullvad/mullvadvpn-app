package net.mullvad.mullvadvpn.lib.ui.component.dialog

import android.os.Parcelable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar

@Preview
@Composable
private fun PreviewInfoConfirmationDialog() {
    AppTheme {
        InfoConfirmationDialog(
            onResult = {},
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

@Parcelize data object Confirmed : Parcelable

@Composable
fun InfoConfirmationDialog(
    onResult: (Confirmed?) -> Unit,
    titleType: InfoConfirmationDialogTitleType,
    confirmButtonTitle: String,
    cancelButtonTitle: String,
    content: @Composable (() -> Unit)? = null,
) {
    InfoConfirmationDialog(
        onResult = onResult,
        confirmValue = Confirmed,
        titleType = titleType,
        confirmButtonTitle = confirmButtonTitle,
        cancelButtonTitle = cancelButtonTitle,
        content = content,
    )
}

@Composable
fun <T> InfoConfirmationDialog(
    onResult: (T?) -> Unit,
    confirmValue: T,
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
        onDismissRequest = { onResult(null) },
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
                        imageVector = Icons.Rounded.Error,
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
                    onClick = { onResult(confirmValue) },
                )

                PrimaryButton(
                    modifier = Modifier.fillMaxWidth(),
                    text = cancelButtonTitle,
                    onClick = { onResult(null) },
                )
            }
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
    )
}
