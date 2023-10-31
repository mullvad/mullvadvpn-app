@file:OptIn(ExperimentalMaterial3Api::class)

package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MediumTopAppBar
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.TopAppBarScrollBehavior
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar

@Preview
@Composable
private fun PreviewTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.inversePrimary,
            iconTintColor = MaterialTheme.colorScheme.onPrimary,
            onSettingsClicked = null,
            onAccountClicked = {}
        )
    }
}

@Preview(widthDp = 260)
@Composable
private fun PreviewSlimTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.inversePrimary,
            iconTintColor = MaterialTheme.colorScheme.onPrimary,
            onSettingsClicked = null,
            onAccountClicked = {}
        )
    }
}

@Preview
@Composable
private fun PreviewNoIconAndLogoTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.inversePrimary,
            iconTintColor = MaterialTheme.colorScheme.onPrimary,
            isIconAndLogoVisible = false,
            onSettingsClicked = {},
            onAccountClicked = null,
        )
    }
}

@Preview
@Composable
private fun PreviewNothingTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.inversePrimary,
            iconTintColor = MaterialTheme.colorScheme.onPrimary,
            isIconAndLogoVisible = false,
            onSettingsClicked = null,
            onAccountClicked = null
        )
    }
}

@Composable
fun MullvadTopBar(
    containerColor: Color,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    modifier: Modifier = Modifier,
    iconTintColor: Color,
    isIconAndLogoVisible: Boolean = true
) {
    TopAppBar(
        modifier = modifier,
        title = {
            if (isIconAndLogoVisible) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        painter = painterResource(id = R.drawable.logo_icon),
                        contentDescription = null, // No meaningful user info or action.
                        modifier = Modifier.size(40.dp),
                        tint = Color.Unspecified
                    )
                    // Dynamically show Mullvad VPN Text if it fits, to avoid overlapping icons.
                    BoxWithConstraints {
                        val logoTextPainter = painterResource(id = R.drawable.logo_text)
                        val logoHeight = Dimens.mediumPadding
                        val logoStartEndPadding = Dimens.mediumPadding

                        val shouldShowText =
                            remember(maxWidth) {
                                val logoHeightWidthRatio =
                                    logoTextPainter.intrinsicSize.width /
                                        logoTextPainter.intrinsicSize.height
                                val expectedLength = logoHeightWidthRatio * logoHeight.value
                                maxWidth > (expectedLength + logoStartEndPadding.value * 2).dp
                            }

                        if (shouldShowText) {
                            Icon(
                                painter = painterResource(id = R.drawable.logo_text),
                                tint = iconTintColor,
                                contentDescription = null, // No meaningful user info or action.
                                modifier =
                                    Modifier.padding(horizontal = Dimens.mediumPadding)
                                        .height(logoHeight)
                            )
                        }
                    }
                }
            }
        },
        actions = {
            if (onAccountClicked != null) {
                IconButton(onClick = onAccountClicked) {
                    Icon(
                        painter = painterResource(R.drawable.icon_account),
                        tint = iconTintColor,
                        contentDescription = stringResource(id = R.string.settings_account),
                    )
                }
            }

            if (onSettingsClicked != null) {
                IconButton(onClick = onSettingsClicked) {
                    Icon(
                        painter = painterResource(R.drawable.icon_settings),
                        tint = iconTintColor,
                        contentDescription = stringResource(id = R.string.settings),
                    )
                }
            }
        },
        colors =
            TopAppBarDefaults.topAppBarColors(
                containerColor = containerColor,
                actionIconContentColor = iconTintColor,
            ),
    )
}

@Preview
@Composable
private fun PreviewMediumTopBar() {
    AppTheme {
        MullvadMediumTopBar(
            title = "Title",
        )
    }
}

@Preview(widthDp = 260)
@Composable
private fun PreviewSlimMediumTopBar() {
    AppTheme {
        MullvadMediumTopBar(
            title = "Long top bar with long title",
            actions = {
                IconButton(onClick = {}) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_settings),
                        contentDescription = null
                    )
                }
            }
        )
    }
}

@Composable
fun MullvadMediumTopBar(
    title: String,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    scrollBehavior: TopAppBarScrollBehavior? = null
) {
    MediumTopAppBar(
        title = { Text(text = title, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        navigationIcon = navigationIcon,
        scrollBehavior = scrollBehavior,
        colors =
            TopAppBarDefaults.mediumTopAppBarColors(
                containerColor = MaterialTheme.colorScheme.background,
                actionIconContentColor = MaterialTheme.colorScheme.onPrimary.copy(AlphaTopBar),
            ),
        actions = actions
    )
}

@Preview
@Composable
private fun PreviewMullvadTopBarWithLongDeviceName() {
    AppTheme {
        Surface {
            MullvadTopBarWithDeviceName(
                containerColor = MaterialTheme.colorScheme.error,
                iconTintColor = MaterialTheme.colorScheme.onError,
                onSettingsClicked = null,
                onAccountClicked = null,
                deviceName = "Superstitious Hippopotamus with extra weight",
                daysLeftUntilExpiry = 1
            )
        }
    }
}

@Preview
@Composable
private fun PreviewMullvadTopBarWithShortDeviceName() {
    AppTheme {
        Surface {
            MullvadTopBarWithDeviceName(
                containerColor = MaterialTheme.colorScheme.error,
                iconTintColor = MaterialTheme.colorScheme.onError,
                onSettingsClicked = null,
                onAccountClicked = null,
                deviceName = "Fit Ant",
                daysLeftUntilExpiry = 1
            )
        }
    }
}

@Composable
fun MullvadTopBarWithDeviceName(
    containerColor: Color,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    iconTintColor: Color,
    isIconAndLogoVisible: Boolean = true,
    deviceName: String?,
    daysLeftUntilExpiry: Int?
) {
    Column {
        MullvadTopBar(
            containerColor,
            onSettingsClicked,
            onAccountClicked,
            Modifier,
            iconTintColor,
            isIconAndLogoVisible,
        )

        // Align animation of extra row with the rest of the Topbar
        val appBarContainerColor by
            animateColorAsState(
                targetValue = containerColor,
                animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
                label = "ColorAnimation"
            )
        Row(
            modifier =
                Modifier.background(appBarContainerColor)
                    .padding(
                        bottom = Dimens.smallPadding,
                        start = Dimens.mediumPadding,
                        end = Dimens.mediumPadding
                    )
                    .fillMaxWidth()
                    .animateContentSize(),
            horizontalArrangement = Arrangement.spacedBy(Dimens.mediumPadding)
        ) {
            Text(
                modifier = Modifier.weight(1f, fill = false),
                text =
                    deviceName?.let {
                        stringResource(id = R.string.top_bar_device_name, deviceName)
                    }
                        ?: "",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaTopBar)
            )
            if (daysLeftUntilExpiry != null) {
                Text(
                    text =
                        stringResource(
                            id = R.string.top_bar_time_left,
                            pluralStringResource(
                                id = R.plurals.days,
                                daysLeftUntilExpiry,
                                daysLeftUntilExpiry
                            )
                        ),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaTopBar)
                )
            } else {
                Spacer(Modifier)
            }
        }
    }
}
