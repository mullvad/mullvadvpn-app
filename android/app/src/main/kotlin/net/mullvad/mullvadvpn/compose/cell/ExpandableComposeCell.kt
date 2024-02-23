package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

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
        headlineContent = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.titleMedium,
                modifier = titleModifier.weight(1f, fill = true)
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
            IconButton(
                modifier =
                    Modifier.padding(horizontal = Dimens.miniPadding)
                        .align(Alignment.CenterVertically),
                onClick = onInfoClicked
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_info),
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimary
                )
            }
        }

        ChevronView(
            isExpanded = isExpanded,
            modifier = Modifier.size(Dimens.expandableCellChevronSize)
        )
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
        color = MaterialTheme.colorScheme.onSecondary,
        modifier = modifier
    )
}
