package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadSwitch
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled

@Preview
@Composable
private fun PreviewSwitchComposeCell() {
    AppTheme {
        SpacedColumn(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            HeaderSwitchComposeCell(
                title = "Checkbox Title",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {},
            )
            NormalSwitchComposeCell(
                title = "Checkbox Item",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {},
            )
        }
    }
}

@Composable
fun NormalSwitchComposeCell(
    title: String,
    isToggled: Boolean,
    startPadding: Dp = Dimens.indentedCellStartPadding,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.surfaceContainerLow,
    onBackground: Color = MaterialTheme.colorScheme.onSurface,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    SwitchComposeCell(
        titleView = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.labelLarge,
                modifier = Modifier.weight(1f, true),
                textColor = if (isEnabled) onBackground else onBackground.copy(AlphaDisabled),
            )
        },
        isToggled = isToggled,
        startPadding = startPadding,
        isEnabled = isEnabled,
        background = background,
        onBackground = onBackground,
        onCellClicked = onCellClicked,
        onInfoClicked = onInfoClicked,
    )
}

@Composable
fun HeaderSwitchComposeCell(
    title: String,
    isToggled: Boolean,
    modifier: Modifier = Modifier,
    startPadding: Dp = Dimens.cellStartPadding,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.primary,
    onBackground: Color = MaterialTheme.colorScheme.onPrimary,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    SwitchComposeCell(
        titleView = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.weight(1f, fill = true),
                textColor = onBackground,
            )
        },
        isToggled = isToggled,
        startPadding = startPadding,
        isEnabled = isEnabled,
        background = background,
        onBackground = onBackground,
        onCellClicked = onCellClicked,
        onInfoClicked = onInfoClicked,
        modifier,
    )
}

@Composable
private fun SwitchComposeCell(
    titleView: @Composable RowScope.() -> Unit,
    isToggled: Boolean,
    startPadding: Dp,
    isEnabled: Boolean,
    background: Color,
    onBackground: Color,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    BaseCell(
        modifier = modifier.focusProperties { canFocus = false },
        headlineContent = titleView,
        isRowEnabled = isEnabled,
        bodyView = {
            SwitchCellView(
                onSwitchClicked = onCellClicked,
                isEnabled = isEnabled,
                isToggled = isToggled,
                iconColor = onBackground,
                onInfoClicked = onInfoClicked,
            )
        },
        background = background,
        onCellClicked = { onCellClicked(!isToggled) },
        startPadding = startPadding,
    )
}

@Composable
fun SwitchCellView(
    isEnabled: Boolean,
    isToggled: Boolean,
    iconColor: Color,
    modifier: Modifier = Modifier,
    onSwitchClicked: ((Boolean) -> Unit)? = null,
    onInfoClicked: (() -> Unit)? = null,
) {
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onInfoClicked != null) {
            IconButton(
                modifier =
                    Modifier.align(Alignment.CenterVertically)
                        .padding(horizontal = Dimens.miniPadding),
                onClick = onInfoClicked,
            ) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = stringResource(id = R.string.more_information),
                    tint = iconColor,
                )
            }
        }

        MullvadSwitch(checked = isToggled, onCheckedChange = onSwitchClicked, enabled = isEnabled)
    }
}

@Composable
fun CustomDnsCellSubtitle(isCellClickable: Boolean, modifier: Modifier) {
    val text =
        if (isCellClickable) {
            textResource(id = R.string.custom_dns_footer)
        } else {
            textResource(
                id = R.string.custom_dns_disable_mode_subtitle,
                textResource(id = R.string.dns_content_blockers),
            )
        }
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier,
    )
}

@Composable
fun SwitchComposeSubtitleCell(
    text: String,
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    BaseSubtitleCell(text = text, modifier = modifier, color = color)
}

@Composable
fun SwitchComposeSubtitleCell(
    text: AnnotatedString,
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    BaseSubtitleCell(text = text, modifier = modifier, color = color)
}
