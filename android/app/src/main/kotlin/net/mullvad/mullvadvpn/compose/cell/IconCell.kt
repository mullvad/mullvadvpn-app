package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewIconCell() {
    AppTheme { IconCell(imageVector = Icons.Default.Add, title = "Add") }
}

@Composable
fun IconCell(
    imageVector: ImageVector?,
    title: String,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    titleStyle: TextStyle = MaterialTheme.typography.labelLarge,
    titleColor: Color = MaterialTheme.colorScheme.onPrimary,
    onClick: () -> Unit = {},
    background: Color = MaterialTheme.colorScheme.primary,
    enabled: Boolean = true,
) {
    BaseCell(
        headlineContent = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                imageVector?.let {
                    Icon(
                        imageVector = imageVector,
                        contentDescription = contentDescription,
                        tint = titleColor,
                    )
                    Spacer(modifier = Modifier.width(Dimens.mediumPadding))
                }
                BaseCellTitle(title = title, style = titleStyle, textColor = titleColor)
            }
        },
        onCellClicked = onClick,
        background = background,
        isRowEnabled = enabled,
        modifier = modifier,
    )
}
