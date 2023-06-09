package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.core.text.HtmlCompat
import androidx.core.text.HtmlCompat.FROM_HTML_MODE_COMPACT
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CellSwitch
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewSwitchComposeCell() {
    AppTheme {
        SpacedColumn {
            HeaderSwitchComposeCell(
                title = "Checkbox Title",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {}
            )
            HeaderSwitchComposeCell(
                title = "Checkbox Title",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {},
                subtitle = "Subtitle"
            )
            NormalSwitchComposeCell(
                title = "Checkbox Item",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {}
            )
        }
    }
}

@Composable
fun NormalSwitchComposeCell(
    title: String,
    isToggled: Boolean,
    startPadding: Dp = Dimens.indentedCellStartPadding,
    subtitle: String? = null,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.primary,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    SwitchComposeCell(
        titleView = { BaseCellTitle(title = title, style = MaterialTheme.typography.labelLarge) },
        isToggled = isToggled,
        startPadding = startPadding,
        subtitle = subtitle,
        isEnabled = isEnabled,
        background = background,
        onCellClicked = onCellClicked,
        onInfoClicked = onInfoClicked
    )
}

@Composable
fun HeaderSwitchComposeCell(
    title: String,
    isToggled: Boolean,
    startPadding: Dp = Dimens.cellStartPadding,
    subtitle: String? = null,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.primary,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    SwitchComposeCell(
        titleView = { BaseCellTitle(title = title, style = MaterialTheme.typography.titleMedium) },
        isToggled = isToggled,
        startPadding = startPadding,
        subtitle = subtitle,
        isEnabled = isEnabled,
        background = background,
        onCellClicked = onCellClicked,
        onInfoClicked = onInfoClicked
    )
}

@Composable
private fun SwitchComposeCell(
    titleView: @Composable () -> Unit,
    isToggled: Boolean,
    startPadding: Dp,
    subtitle: String?,
    isEnabled: Boolean,
    background: Color,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)?
) {
    BaseCell(
        title = titleView,
        subtitle =
            subtitle?.let {
                @Composable {
                    Text(
                        text = it,
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSecondary
                    )
                }
            },
        isRowEnabled = isEnabled,
        bodyView = {
            SwitchCellView(
                onSwitchClicked = null,
                isEnabled = isEnabled,
                isToggled = isToggled,
                onInfoClicked = onInfoClicked
            )
        },
        background = background,
        onCellClicked = { onCellClicked(!isToggled) },
        startPadding = startPadding
    )
}

@Composable
fun SwitchCellView(
    isEnabled: Boolean,
    isToggled: Boolean,
    modifier: Modifier = Modifier,
    onSwitchClicked: ((Boolean) -> Unit)? = null,
    onInfoClicked: (() -> Unit)? = null
) {
    val horizontalPadding = Dimens.mediumPadding
    val verticalPadding = 13.dp
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (onInfoClicked != null) {
            Icon(
                modifier =
                    Modifier.clickable { onInfoClicked() }
                        .padding(
                            start = horizontalPadding,
                            end = horizontalPadding,
                            top = verticalPadding,
                            bottom = verticalPadding
                        )
                        .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onPrimary
            )
        }

        CellSwitch(isChecked = isToggled, isEnabled = isEnabled, onCheckedChange = onSwitchClicked)
    }
}

@Composable
fun CustomDnsCellSubtitle(isCellClickable: Boolean, modifier: Modifier) {
    val spanned =
        HtmlCompat.fromHtml(
            if (isCellClickable) {
                textResource(id = R.string.custom_dns_footer)
            } else {
                textResource(
                    id = R.string.custom_dns_disable_mode_subtitle,
                    textResource(id = R.string.dns_content_blockers_title)
                )
            },
            FROM_HTML_MODE_COMPACT
        )
    Text(
        text = spanned.toAnnotatedString(boldFontWeight = FontWeight.ExtraBold),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSecondary,
        modifier = modifier
    )
}
