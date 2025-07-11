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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LargeTopAppBar
import androidx.compose.material3.LocalTextStyle
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_ACCOUNT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_SETTINGS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_TEST_TAG

@Preview
@Composable
private fun PreviewTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.tertiary,
            iconTintColor = MaterialTheme.colorScheme.onTertiary,
            onSettingsClicked = null,
            onAccountClicked = {},
        )
    }
}

@Preview(widthDp = 260)
@Composable
private fun PreviewSlimTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.tertiary,
            iconTintColor = MaterialTheme.colorScheme.onTertiary,
            onSettingsClicked = null,
            onAccountClicked = {},
        )
    }
}

@Preview
@Composable
private fun PreviewNoIconAndLogoTopBar() {
    AppTheme {
        MullvadTopBar(
            containerColor = MaterialTheme.colorScheme.tertiary,
            iconTintColor = MaterialTheme.colorScheme.onTertiary,
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
            containerColor = MaterialTheme.colorScheme.tertiary,
            iconTintColor = MaterialTheme.colorScheme.onTertiary,
            isIconAndLogoVisible = false,
            onSettingsClicked = null,
            onAccountClicked = null,
        )
    }
}

@Composable
fun MullvadTopBar(
    containerColor: Color,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    iconTintColor: Color,
    isIconAndLogoVisible: Boolean = true,
) {
    TopAppBar(
        modifier = modifier.testTag(TOP_BAR_TEST_TAG),
        title = {
            if (isIconAndLogoVisible) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        painter = painterResource(id = R.drawable.logo_icon),
                        contentDescription = null, // No meaningful user info or action.
                        modifier = Modifier.size(40.dp),
                        tint = Color.Unspecified, // Logo should not be tinted
                    )
                    // Dynamically show Mullvad VPN Text if it fits, to avoid overlapping icons.
                    BoxWithConstraints {
                        val logoTextPainter = painterResource(id = R.drawable.logo_text)
                        val logoHeight = Dimens.mediumPadding
                        val logoStartEndPadding = Dimens.smallPadding

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
                                    Modifier.padding(horizontal = logoStartEndPadding)
                                        .height(logoHeight),
                            )
                        }
                    }
                }
            }
        },
        actions = {
            if (onAccountClicked != null) {
                IconButton(
                    modifier = Modifier.testTag(TOP_BAR_ACCOUNT_BUTTON_TEST_TAG),
                    enabled = enabled,
                    onClick = onAccountClicked,
                ) {
                    Icon(
                        imageVector = Icons.Default.AccountCircle,
                        tint = iconTintColor,
                        contentDescription = stringResource(id = R.string.settings_account),
                    )
                }
            }

            if (onSettingsClicked != null) {
                IconButton(
                    modifier = Modifier.testTag(TOP_BAR_SETTINGS_BUTTON_TEST_TAG),
                    enabled = enabled,
                    onClick = onSettingsClicked,
                ) {
                    Icon(
                        imageVector = Icons.Default.Settings,
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

@Composable
fun MullvadSmallTopBar(
    title: String,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
) {
    TopAppBar(
        title = { Text(text = title, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        navigationIcon = navigationIcon,
        colors =
            TopAppBarDefaults.topAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface,
                scrolledContainerColor = MaterialTheme.colorScheme.surface,
                actionIconContentColor = MaterialTheme.colorScheme.onSurface,
            ),
        actions = actions,
    )
}

@Preview
@Composable
private fun PreviewMediumTopBar() {
    AppTheme { MullvadMediumTopBar(title = "Title") }
}

@Preview
@Composable
private fun PreviewLargeTopBar() {
    AppTheme { MullvadLargeTopBar(title = "Title") }
}

@Preview(widthDp = 260)
@Composable
private fun PreviewSlimMediumTopBar() {
    AppTheme {
        MullvadMediumTopBar(
            title = "Long top bar with long title",
            actions = {
                IconButton(onClick = {}) {
                    Icon(imageVector = Icons.Default.Settings, contentDescription = null)
                }
            },
        )
    }
}

@Composable
fun MullvadMediumTopBar(
    title: String,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    scrollBehavior: TopAppBarScrollBehavior? = null,
) {
    MediumTopAppBar(
        title = { Text(text = title, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        navigationIcon = navigationIcon,
        scrollBehavior = scrollBehavior,
        colors =
            TopAppBarDefaults.mediumTopAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface,
                scrolledContainerColor = MaterialTheme.colorScheme.surface,
                actionIconContentColor = MaterialTheme.colorScheme.onSurface,
            ),
        actions = actions,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MullvadLargeTopBar(
    title: String,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    scrollBehavior: TopAppBarScrollBehavior? = null,
) {
    LargeTopAppBar(
        title = {
            Text(
                text = title,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = LocalTextStyle.current.copy(lineBreak = LineBreak.Heading),
            )
        },
        navigationIcon = navigationIcon,
        scrollBehavior = scrollBehavior,
        colors =
            TopAppBarDefaults.mediumTopAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface,
                scrolledContainerColor = MaterialTheme.colorScheme.surface,
                actionIconContentColor = MaterialTheme.colorScheme.onSurface,
            ),
        actions = actions,
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
                daysLeftUntilExpiry = 1,
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
                daysLeftUntilExpiry = 1,
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
    daysLeftUntilExpiry: Long?,
) {
    Column {
        MullvadTopBar(
            containerColor,
            onSettingsClicked,
            onAccountClicked,
            Modifier,
            enabled = true,
            iconTintColor,
            isIconAndLogoVisible,
        )

        // Align animation of extra row with the rest of the Topbar
        val appBarContainerColor by
            animateColorAsState(
                targetValue = containerColor,
                animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
                label = "ColorAnimation",
            )
        Row(
            modifier =
                Modifier.background(appBarContainerColor)
                    .padding(
                        bottom = Dimens.smallPadding,
                        start = Dimens.mediumPadding,
                        end = Dimens.mediumPadding,
                    )
                    .fillMaxWidth()
                    .animateContentSize(),
            horizontalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
        ) {
            Text(
                modifier = Modifier.weight(1f, fill = false),
                text =
                    deviceName?.let {
                        stringResource(id = R.string.top_bar_device_name, deviceName)
                    } ?: "",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.labelLarge,
                color = iconTintColor,
            )
            if (daysLeftUntilExpiry != null) {
                Text(
                    text =
                        stringResource(
                            id = R.string.top_bar_time_left,
                            if (daysLeftUntilExpiry >= 0) {
                                pluralStringResource(
                                    id = R.plurals.days,
                                    daysLeftUntilExpiry.toInt(),
                                    daysLeftUntilExpiry.toInt(),
                                )
                            } else {
                                stringResource(id = R.string.out_of_time)
                            },
                        ),
                    style = MaterialTheme.typography.labelLarge,
                    color = iconTintColor,
                )
            } else {
                Spacer(Modifier)
            }
        }
    }
}
