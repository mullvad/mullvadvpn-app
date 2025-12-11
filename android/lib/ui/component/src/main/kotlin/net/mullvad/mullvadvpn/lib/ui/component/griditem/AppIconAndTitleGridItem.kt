package net.mullvad.mullvadvpn.lib.ui.component.griditem

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull

@Preview
@Composable
private fun PreviewAppIconAndTitleGridItem() {
    AppTheme {
        FlowRow(Modifier.background(MaterialTheme.colorScheme.surface)) {
            AppIconAndTitleGridItem(
                appTitle = "Obfuscation",
                appIcon = R.drawable.notes_preview,
                isSelected = true,
                onClick = {},
            )
            AppIconAndTitleGridItem(
                appTitle = "Obfuscation",
                appIcon = R.drawable.weather_preview,
                isSelected = true,
                onClick = {},
            )
        }
    }
}

@Composable
fun AppIconAndTitleGridItem(
    modifier: Modifier = Modifier,
    appTitle: String,
    appIcon: Int,
    isSelected: Boolean,
    appIconContentDescription: String? = null,
    onClick: (() -> Unit),
    testTag: String? = null,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = modifier.applyIfNotNull(testTag) { testTag(it) }.clickable(onClick = onClick),
    ) {
        Icon(
            painter = painterResource(appIcon),
            contentDescription = appIconContentDescription,
            modifier = Modifier.padding(top = INNER_PADDING).size(APP_ICON_SIZE),
            tint = Color.Unspecified,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
        Row(
            modifier = Modifier.padding(horizontal = INNER_PADDING).padding(bottom = INNER_PADDING),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center,
        ) {
            AnimatedVisibility(
                // modifier = Modifier.align(Alignment.Center),
                modifier = Modifier.align(Alignment.CenterVertically),
                visible = isSelected,
                enter =
                    fadeIn(tween(ANIMATION_DURATION)) +
                        expandHorizontally(tween(ANIMATION_DURATION)),
                exit =
                    fadeOut(tween(ANIMATION_DURATION)) +
                        shrinkHorizontally(tween(ANIMATION_DURATION)),
            ) {
                Icon(
                    modifier = Modifier.padding(end = Dimens.smallPadding),
                    imageVector = Icons.Default.Check,
                    contentDescription = null,
                    // Set the tint explicitly here because the animation looks better if the icon
                    // does not change color to white while sliding out.
                    tint = MaterialTheme.colorScheme.tertiary,
                )
            }
            Text(
                text = appTitle,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

private val APP_ICON_SIZE = 32.dp
private val INNER_PADDING = 4.dp
private const val ANIMATION_DURATION = 200
