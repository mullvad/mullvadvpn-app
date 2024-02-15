package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
fun PreviewTwoRowCell() {
    AppTheme { TwoRowCell(titleText = "Title", subtitleText = "Subtitle") }
}

@Composable
fun TwoRowCell(
    titleText: String,
    subtitleText: String,
    onCellClicked: () -> Unit = {},
    titleColor: Color = MaterialTheme.colorScheme.onPrimary,
    subtitleColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary
) {
    BaseCell(
        title = {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = titleText,
                    style = MaterialTheme.typography.labelLarge,
                    color = titleColor
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = subtitleText,
                    style = MaterialTheme.typography.labelLarge,
                    color = subtitleColor
                )
            }
        },
        onCellClicked = onCellClicked,
        background = background,
        minHeight = 72.dp
    )
}
