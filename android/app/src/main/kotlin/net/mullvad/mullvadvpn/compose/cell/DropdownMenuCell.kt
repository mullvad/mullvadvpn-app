package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewThreeDotCell() {
    AppTheme { ThreeDotCell(text = "Three dots", onClickDots = {}) }
}

@Composable
fun ThreeDotCell(
    text: String,
    modifier: Modifier = Modifier,
    onClickDots: () -> Unit,
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary,
) {
    BaseCell(
        headlineContent = {
            BaseCellTitle(
                title = text,
                style = textStyle,
                textColor = textColor,
                modifier = Modifier.weight(1f, true),
            )
        },
        modifier = modifier,
        background = background,
        bodyView = {
            IconButton(onClick = onClickDots) {
                Icon(
                    imageVector = Icons.Default.MoreVert,
                    contentDescription = stringResource(id = R.string.custom_lists),
                    tint = textColor,
                )
            }
        },
        isRowEnabled = false,
        endPadding = Dimens.smallPadding,
    )
}
