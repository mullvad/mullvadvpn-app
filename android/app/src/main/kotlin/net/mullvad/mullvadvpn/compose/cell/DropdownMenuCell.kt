package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.layout
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
fun PreviewDropdownMenuCell() {
    AppTheme {
        DropdownMenuCell(
            text = "Dropdown Menu",
            contextMenuItems = {
                DropdownMenuItem({ Text("Item1") }, {})
                DropdownMenuItem({ Text("Item1") }, {})
                DropdownMenuItem({ Text("Item1") }, {})
            }
        )
    }
}

@Composable
fun DropdownMenuCell(
    text: String,
    contextMenuItems: @Composable (onClose: () -> Unit) -> Unit,
    modifier: Modifier = Modifier,
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary,
    dropdownBackground: Color = MaterialTheme.colorScheme.background,
    dropdownBorderColor: Color = MaterialTheme.colorScheme.primary
) {
    var showMenu by remember { mutableStateOf(false) }
    BaseCell(
        headlineContent = {
            BaseCellTitle(
                title = text,
                style = textStyle,
                color = textColor,
                modifier = Modifier.weight(1f, true)
            )
        },
        modifier = modifier,
        background = background,
        bodyView = {
            IconButton(onClick = { showMenu = true }) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_more_vert),
                    contentDescription = null
                )
                if (showMenu) {
                    DropdownMenu(
                        expanded = true,
                        onDismissRequest = { showMenu = false },
                        modifier =
                            Modifier.background(dropdownBackground)
                                .border(
                                    width = Dimens.dropdownMenuBorder,
                                    color = dropdownBorderColor,
                                    MaterialTheme.shapes.extraSmall
                                )
                                .crop(vertical = Dimens.dropdownMenuVerticalPadding)
                    ) {
                        contextMenuItems { showMenu = false }
                    }
                }
            }
        },
        isRowEnabled = false,
        endPadding = Dimens.smallPadding
    )
}

fun Modifier.crop(
    horizontal: Dp = 0.dp,
    vertical: Dp = 0.dp,
): Modifier =
    this.layout { measurable, constraints ->
        val placeable = measurable.measure(constraints)
        fun Dp.toPxInt(): Int = this.toPx().toInt()

        layout(
            placeable.width - (horizontal * 2).toPxInt(),
            placeable.height - (vertical * 2).toPxInt()
        ) {
            placeable.placeRelative(-horizontal.toPx().toInt(), -vertical.toPx().toInt())
        }
    }
