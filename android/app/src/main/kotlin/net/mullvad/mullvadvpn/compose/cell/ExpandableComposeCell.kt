package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ExpandChevronIconButton
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
            onInfoClicked = {},
        )
    }
}

@Composable
fun ExpandableComposeCell(
    title: String,
    isExpanded: Boolean,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    testTag: String = "",
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    background: Color = MaterialTheme.colorScheme.primary,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    val titleModifier = Modifier.alpha(if (isEnabled) AlphaVisible else AlphaInactive)
    val bodyViewModifier = Modifier

    BaseCell(
        modifier = modifier.testTag(testTag).focusProperties { canFocus = false },
        headlineContent = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.titleMedium,
                textColor = textColor,
                modifier = titleModifier.weight(1f, fill = true),
            )
        },
        bodyView = {
            ExpandableComposeCellBody(
                isExpanded = isExpanded,
                modifier = bodyViewModifier,
                onExpand = onCellClicked,
                onInfoClicked = onInfoClicked,
            )
        },
        background = background,
        onCellClicked = { onCellClicked(!isExpanded) },
    )
}

@Composable
private fun ExpandableComposeCellBody(
    isExpanded: Boolean,
    modifier: Modifier,
    onExpand: ((Boolean) -> Unit),
    onInfoClicked: (() -> Unit)? = null,
) {
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onInfoClicked != null) {
            IconButton(
                modifier =
                    Modifier.padding(horizontal = Dimens.miniPadding)
                        .align(Alignment.CenterVertically),
                onClick = onInfoClicked,
            ) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = stringResource(id = R.string.more_information),
                    tint = MaterialTheme.colorScheme.onPrimary,
                )
            }
        }

        ExpandChevronIconButton(
            isExpanded = isExpanded,
            onExpand = onExpand,
            color = MaterialTheme.colorScheme.onPrimary,
        )
    }
}

@Composable
fun ContentBlockersDisableModeCellSubtitle(modifier: Modifier) {
    BaseSubtitleCell(
        text =
            stringResource(
                id = R.string.dns_content_blockers_subtitle,
                stringResource(id = R.string.enable_custom_dns),
            ),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier,
    )
}
