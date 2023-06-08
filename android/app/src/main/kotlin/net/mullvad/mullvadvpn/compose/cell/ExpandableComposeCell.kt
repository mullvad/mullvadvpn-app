package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ChevronView
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.AlphaVisible
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewExpandedEnabledExpandableComposeCell() {
    AppTheme {
        ExpandableComposeCell(
            title = "Expandable row title",
            isExpanded = true,
            isEnabled = true,
            onCellClicked = {},
            onInfoClicked = {}
        )
    }
}

@Composable
fun ExpandableComposeCell(
    title: String,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val titleModifier = Modifier.alpha(if (isEnabled) AlphaVisible else AlphaInactive)
    val bodyViewModifier = Modifier

    BaseCell(
        title = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.titleMedium,
                modifier = titleModifier
            )
        },
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
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (onInfoClicked != null) {
            Icon(
                modifier =
                    Modifier.clickable { onInfoClicked() }
                        .padding(
                            start = Dimens.mediumPadding,
                            end = Dimens.mediumPadding,
                            top = Dimens.infoButtonVerticalPadding,
                            bottom = Dimens.infoButtonVerticalPadding
                        )
                        .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onPrimary
            )
        }

        ChevronView(isExpanded)
    }
}

@Composable
fun ContentBlockersDisableModeCellSubtitle(modifier: Modifier) {
    val spanned =
        HtmlCompat.fromHtml(
            textResource(
                id = R.string.dns_content_blockers_subtitle,
                stringResource(id = R.string.enable_custom_dns)
            ),
            HtmlCompat.FROM_HTML_MODE_COMPACT
        )
    Text(
        text = spanned.toAnnotatedString(boldFontWeight = FontWeight.ExtraBold),
        style = MaterialTheme.typography.labelMedium,
        modifier = modifier
    )
}
