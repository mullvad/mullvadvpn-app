package net.mullvad.mullvadvpn.compose.component

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Preview
@Composable
private fun PreviewListItem() {
    AppTheme {
        Column {
            ListItem(text = "No subtext No icon not loading", isLoading = false, onClick = {})
            ListItem(text = "No subtext No icon is loading", isLoading = true, onClick = {})
            ListItem(
                text = "No subtext With icon is loading",
                isLoading = true,
                iconResourceId = R.drawable.icon_close,
                onClick = {}
            )
            ListItem(
                text = "No subtext With icon not loading",
                isLoading = false,
                iconResourceId = R.drawable.icon_close,
                onClick = {}
            )
            ListItem(
                text = "With subtext with icon is loading",
                subText = "Subtext",
                isLoading = true,
                iconResourceId = R.drawable.icon_close,
                onClick = {}
            )
            ListItem(
                text = "With subtext no icon is loading",
                subText = "Subtext",
                isLoading = true,
                onClick = {}
            )
            ListItem(
                text = "With subtext with icon not loading",
                subText = "Subtext",
                isLoading = false,
                iconResourceId = R.drawable.icon_close,
                onClick = {}
            )
            ListItem(
                text = "With subtext no icon not loading",
                subText = "Subtext",
                isLoading = false,
                onClick = {}
            )
        }
    }
}

@Composable
fun ListItem(
    text: String,
    subText: String? = null,
    height: Dp = Dimens.listItemHeight,
    isLoading: Boolean,
    @DrawableRes iconResourceId: Int? = null,
    background: Color = MaterialTheme.colorScheme.primary,
    onClick: (() -> Unit)?
) {
    Box(
        modifier =
            Modifier.fillMaxWidth()
                .padding(vertical = Dimens.listItemDivider)
                .wrapContentHeight()
                .defaultMinSize(minHeight = height)
                .background(background)
    ) {
        Column(
            modifier =
                Modifier.padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding)
                    .align(Alignment.CenterStart)
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.listItemText,
                color = MaterialTheme.colorScheme.onPrimary
            )
            subText?.let {
                Text(
                    text = subText,
                    style = MaterialTheme.typography.listItemSubText,
                    color =
                        MaterialTheme.colorScheme.onPrimary
                            .copy(alpha = AlphaDescription)
                            .compositeOver(background)
                )
            }
        }

        Box(
            modifier =
                Modifier.align(Alignment.CenterEnd)
                    .padding(horizontal = Dimens.loadingSpinnerPadding)
        ) {
            if (isLoading) {
                MullvadCircularProgressIndicatorMedium()
            } else if (iconResourceId != null) {
                Image(
                    painter = painterResource(id = iconResourceId),
                    contentDescription = "Remove",
                    modifier =
                        onClick?.let { Modifier.align(Alignment.CenterEnd).clickable { onClick() } }
                            ?: Modifier.align(Alignment.CenterEnd)
                )
            }
        }
    }
}
