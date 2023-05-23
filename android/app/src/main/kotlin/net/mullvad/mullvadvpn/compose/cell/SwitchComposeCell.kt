package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CellSwitch
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.theme.AlphaActive
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewSwitchComposeCell() {
    AppTheme {
        Column {
            SwitchComposeCell(
                title = "Checkbox Title",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {}
            )
            Spacer(modifier = Modifier.height(1.dp))
            SwitchComposeCell(
                title = "Checkbox Title",
                isEnabled = true,
                isToggled = true,
                onCellClicked = {},
                onInfoClicked = {},
                subtitle = "Subtitle"
            )
            Spacer(modifier = Modifier.height(1.dp))
            SwitchComposeCell(
                title = "Checkbox Item",
                isEnabled = true,
                isToggled = true,
                isHeader = false,
                onCellClicked = {},
                onInfoClicked = {}
            )
        }
    }
}

@Composable
fun SwitchComposeCell(
    title: String,
    isToggled: Boolean,
    startPadding: Dp = Dimens.cellStartPadding,
    isHeader: Boolean = true,
    subtitle: String? = null,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.primary,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    BaseCell(
        title = {
            SwitchCellTitle(
                title = title,
                isHeader = isHeader,
                modifier = Modifier.alpha(if (isEnabled) AlphaActive else AlphaInactive)
            )
        },
        subtitle =
            subtitle?.let {
                @Composable {
                    Text(
                        text = it,
                        style = MaterialTheme.typography.labelMedium,
                        color = MullvadWhite60
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
fun SwitchCellTitle(title: String, isHeader: Boolean, modifier: Modifier = Modifier) {
    Text(
        text = title,
        textAlign = TextAlign.Center,
        style =
            if (isHeader) {
                MaterialTheme.typography.titleMedium
            } else {
                MaterialTheme.typography.labelLarge
            },
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
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
    val textSize = dimensionResource(id = R.dimen.text_small).value

    HtmlText(
        htmlFormattedString =
            textResource(
                if (isCellClickable) {
                    R.string.custom_dns_footer
                } else {
                    R.string.custom_dns_disable_mode_subtitle
                }
            ),
        textSize = textSize,
        textColor = MullvadWhite60.toArgb(),
        modifier = modifier
    )
}
