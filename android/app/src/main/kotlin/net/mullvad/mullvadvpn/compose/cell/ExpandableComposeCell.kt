package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ChevronView
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.theme.AlphaActive
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewExpandedEnabledExpandableComposeCell() {
    ExpandableComposeCell(
        title = "Expandable row title",
        isExpanded = true,
        isEnabled = true,
        onCellClicked = {},
        onInfoClicked = {}
    )
}

@Composable
fun ExpandableComposeCell(
    title: String,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val titleModifier = Modifier.alpha(if (isEnabled) AlphaActive else AlphaInactive)
    val bodyViewModifier = Modifier

    BaseCell(
        title = { SwitchCellTitle(title = title, modifier = titleModifier) },
        bodyView = {
            ExpandableComposeCellBody(
                isExpanded = isExpanded,
                modifier = bodyViewModifier,
                onInfoClicked = onInfoClicked
            )
        },
        onCellClicked = { onCellClicked(!isExpanded) }
    )
}

@Composable
private fun ExpandableComposeCellBody(
    isExpanded: Boolean,
    modifier: Modifier,
    onInfoClicked: (() -> Unit)? = null
) {
    val horizontalPadding = dimensionResource(id = R.dimen.medium_padding)
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
                contentDescription = stringResource(id = R.string.confirm_local_dns),
                tint = MullvadWhite
            )
        }

        ChevronView(isExpanded)
    }
}

@Composable
fun ContentBlockersDisableModeCellSubtitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_small).value

    HtmlText(
        htmlFormattedString =
            textResource(
                id = R.string.dns_content_blockers_subtitle,
                stringResource(id = R.string.enable_custom_dns)
            ),
        textSize = textSize,
        textColor = MullvadWhite60.toArgb(),
        modifier = modifier
    )
}
