package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewTwoRowCell() {
    AppTheme { TwoRowCell(titleText = "Title", subtitleText = "Subtitle") }
}

@Composable
fun TwoRowCell(
    titleText: String,
    subtitleText: String,
    bodyView: @Composable ColumnScope.() -> Unit = {},
    onCellClicked: () -> Unit = {},
    titleColor: Color = MaterialTheme.colorScheme.onPrimary,
    subtitleColor: Color = MaterialTheme.colorScheme.onPrimary,
    titleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    subtitleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    background: Color = MaterialTheme.colorScheme.primary,
) {
    BaseCell(
        headlineContent = {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = titleText,
                    style = titleStyle,
                    color = titleColor,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = subtitleText,
                    style = subtitleStyle,
                    color = subtitleColor,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        },
        bodyView = bodyView,
        onCellClicked = onCellClicked,
        background = background,
        minHeight = Dimens.cellHeightTwoRows,
    )
}
