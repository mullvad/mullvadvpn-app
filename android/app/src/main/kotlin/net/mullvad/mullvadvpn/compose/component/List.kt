package net.mullvad.mullvadvpn.compose.component

import android.content.res.Configuration
import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.absolutePadding
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.dimensions.Dimens
import net.mullvad.mullvadvpn.compose.theme.typeface.listItemText

@Preview
@Composable
fun PreviewListItem() {
    Column {
        ListItem(text = "No icon not loading", isLoading = false, onClick = {})
        ListItem(text = "No icon is loading", isLoading = true, onClick = {})
        ListItem(
            text = "With icon is loading",
            isLoading = true,
            iconResourceId = R.drawable.icon_close,
            onClick = {}
        )
        ListItem(
            text = "With icon not loading",
            isLoading = false,
            iconResourceId = R.drawable.icon_close,
            onClick = {}
        )
    }
}

@Preview
@Composable
fun PreviewChangeListItem() {
    ChangeListItem(text = "ChangeListItem")
}

@Composable
fun ListItem(
    text: String,
    height: Dp = Dimens.listItemHeight,
    isLoading: Boolean,
    @DrawableRes iconResourceId: Int? = null,
    onClick: () -> Unit
) {
    Box(
        modifier =
            Modifier.fillMaxWidth()
                .padding(vertical = Dimens.listItemDivider)
                .wrapContentHeight()
                .defaultMinSize(minHeight = height)
                .background(BackgroundColor),
    ) {
        Column(
            modifier =
                Modifier.padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding)
                    .align(Alignment.CenterStart)
        ) {
            Text(text = text, style = MaterialTheme.typography.listItemText, color = TextColor)
        }

        Box(
            modifier =
                Modifier.align(Alignment.CenterEnd)
                    .padding(horizontal = Dimens.loadingSpinnerPadding)
        ) {
            if (isLoading) {
                CircularProgressIndicator(
                    strokeWidth = Dimens.loadingSpinnerStrokeWidth,
                    color = TextColor,
                    modifier =
                        Modifier.height(Dimens.loadingSpinnerSize).width(Dimens.loadingSpinnerSize)
                )
            } else if (iconResourceId != null) {
                Image(
                    painter = painterResource(id = iconResourceId),
                    contentDescription = "Remove",
                    modifier = Modifier.align(Alignment.CenterEnd).clickable { onClick() }
                )
            }
        }
    }
}

@Composable
fun ChangeListItem(text: String) {
    val smallPadding = Dimens.smallPadding

    ConstraintLayout {
        val (bullet, changeLog) = createRefs()
        Box(
            modifier =
                Modifier.constrainAs(bullet) {
                    top.linkTo(parent.top)
                    start.linkTo(parent.absoluteLeft)
                }
        ) {
            Text(text = "•", style = MaterialTheme.typography.bodyMedium, color = TextColor)
        }
        Box(
            modifier =
                Modifier.absolutePadding(left = Dimens.mediumPadding).constrainAs(changeLog) {
                    top.linkTo(parent.top)
                    bottom.linkTo(parent.bottom, margin = smallPadding)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                color = TextColor,
                modifier = Modifier
            )
        }
    }
}

private val BackgroundColor: Color
    @Composable get() = MaterialTheme.colorScheme.primary

private val TextColor: Color
    @Composable get() = MaterialTheme.colorScheme.onPrimary
