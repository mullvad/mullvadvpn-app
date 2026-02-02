package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewIconCell() {
    AppTheme {
        SpacedColumn {
            IconCell(imageVector = Icons.Default.Add, title = "Add")
            IconCell(
                imageVector = Icons.Default.Remove,
                title = "Remove",
                endIcon = {
                    Icon(
                        imageVector = Icons.Default.Error,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimary,
                    )
                },
            )
        }
    }
}

@Suppress("ComposableLambdaParameterNaming")
@Composable
fun IconCell(
    imageVector: ImageVector?,
    title: String,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    titleStyle: TextStyle = MaterialTheme.typography.bodyLarge,
    titleColor: Color = MaterialTheme.colorScheme.onPrimary,
    onClick: () -> Unit = {},
    enabled: Boolean = true,
    endIcon: @Composable ColumnScope.() -> Unit = {},
) {
    // Using a transparent background to avoid using the default background from BaseCell
    val background: Color = Color.Transparent

    BaseCell(
        headlineContent = {
            Row(modifier = Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically) {
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
        bodyView = endIcon,
        onCellClicked = onClick,
        background = background,
        isRowEnabled = enabled,
        modifier = modifier,
    )
}
