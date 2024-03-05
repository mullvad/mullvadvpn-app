package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

@Preview
@Composable
private fun PreviewInformationComposeCell() {
    AppTheme {
        InformationComposeCell(
            title = "Information row title",
            isEnabled = true,
            onCellClicked = {},
            onInfoClicked = {}
        )
    }
}

@Composable
fun InformationComposeCell(
    title: String,
    isEnabled: Boolean = true,
    background: Color = MaterialTheme.colorScheme.primary,
    onCellClicked: () -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val titleModifier = Modifier.alpha(if (isEnabled) AlphaVisible else AlphaInactive)
    val bodyViewModifier = Modifier

    BaseCell(
        modifier = Modifier.focusProperties { canFocus = false },
        title = {
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.titleMedium,
                modifier = titleModifier.weight(1f, true)
            )
        },
        background = background,
        bodyView = {
            InformationComposeCellBody(modifier = bodyViewModifier, onInfoClicked = onInfoClicked)
        },
        onCellClicked = onCellClicked
    )
}

@Composable
private fun InformationComposeCellBody(modifier: Modifier, onInfoClicked: (() -> Unit)? = null) {
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (onInfoClicked != null) {
            IconButton(
                onClick = onInfoClicked,
                modifier =
                    Modifier.padding(horizontal = Dimens.miniPadding)
                        .align(Alignment.CenterVertically),
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_info),
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimary
                )
            }
        }
    }
}
