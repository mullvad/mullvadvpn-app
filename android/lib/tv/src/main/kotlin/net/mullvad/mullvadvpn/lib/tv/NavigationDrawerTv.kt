package net.mullvad.mullvadvpn.lib.tv

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.FocusRequester.Companion.Cancel
import androidx.compose.ui.focus.FocusRequester.Companion.Default
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import androidx.compose.ui.unit.dp
import androidx.tv.material3.DrawerValue
import androidx.tv.material3.ModalNavigationDrawer
import androidx.tv.material3.NavigationDrawerItem
import androidx.tv.material3.NavigationDrawerItemDefaults
import androidx.tv.material3.NavigationDrawerScope
import androidx.tv.material3.rememberDrawerState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

private class DrawerValueProvider : PreviewParameterProvider<DrawerValue> {
    override val values: Sequence<DrawerValue>
        get() = sequenceOf(DrawerValue.Closed, DrawerValue.Open)
}

@Preview("Closed|Open")
@Composable
fun PreviewNavigationDrawerTvClosed(
    @PreviewParameter(DrawerValueProvider::class) drawerValue: DrawerValue
) {
    AppTheme {
        NavigationDrawerTv(
            daysLeftUntilExpiry = 30,
            deviceName = "Cool Cat",
            initialDrawerValue = drawerValue,
            onSettingsClick = {},
            onAccountClick = {},
        ) {}
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
@Suppress("LongMethod")
fun NavigationDrawerTv(
    daysLeftUntilExpiry: Long?,
    deviceName: String?,
    initialDrawerValue: DrawerValue = DrawerValue.Closed,
    onSettingsClick: (() -> Unit),
    onAccountClick: (() -> Unit),
    content: @Composable () -> Unit,
) {
    val drawerState = rememberDrawerState(initialDrawerValue)
    val focusRequester = remember { FocusRequester() }
    val brush = remember { Brush.horizontalGradient(listOf(Color.Black, Color.Transparent)) }

    val focusManager = LocalFocusManager.current

    if (drawerState.currentValue == DrawerValue.Open) {
        BackHandler(
            onBack = {
                drawerState.setValue(DrawerValue.Closed)
                focusManager.moveFocus(FocusDirection.Right)
            }
        )
    }

    ModalNavigationDrawer(
        modifier =
            Modifier.focusRequester(focusRequester).focusProperties {
                onEnter = { if (focusRequester.restoreFocusedChild()) Cancel else Default }
            },
        drawerState = drawerState,
        scrimBrush = brush,
        drawerContent = {
            Box(
                Modifier.fillMaxHeight()
                    .background(brush)
                    .padding(
                        top = Dimens.screenVerticalMargin,
                        bottom = Dimens.screenVerticalMargin,
                        start = Dimens.tvDrawerHorizontalPadding,
                        end = Dimens.tvDrawerHorizontalPadding,
                    )
                    .selectableGroup()
            ) {
                val animatedPadding =
                    animateDpAsState(
                        if (hasFocus) Dimens.tvDrawerHeaderWithFocusStartPadding
                        else Dimens.tvDrawerHeaderStartPadding
                    )

                NavigationDrawerTvHeader(
                    modifier =
                        Modifier.align(Alignment.TopStart).padding(start = animatedPadding.value),
                    isExpanded = hasFocus,
                    daysLeftUntilExpiry = daysLeftUntilExpiry,
                    deviceName = deviceName,
                )
                DrawerItemTv(
                    modifier =
                        Modifier.align(Alignment.CenterStart).onFocusChanged {
                            focusRequester.saveFocusedChild()
                        },
                    icon = Icons.Default.AccountCircle,
                    text = stringResource(R.string.settings_account),
                    onClick = onAccountClick,
                )
                DrawerItemTv(
                    modifier =
                        Modifier.align(Alignment.BottomStart).onFocusChanged {
                            focusRequester.saveFocusedChild()
                        },
                    icon = Icons.Default.Settings,
                    text = stringResource(R.string.settings),
                    onClick = onSettingsClick,
                )
            }
        },
        content = content,
    )
}

@Composable
private fun NavigationDrawerScope.DrawerItemTv(
    modifier: Modifier = Modifier,
    icon: ImageVector,
    text: String,
    onClick: () -> Unit,
) {
    NavigationDrawerItem(
        modifier = modifier,
        onClick = onClick,
        selected = false,
        leadingContent = {
            Icon(
                tint = MaterialTheme.colorScheme.onPrimary,
                imageVector = icon,
                contentDescription = null,
            )
        },
    ) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.onPrimary,
            text = text,
            maxLines = 1,
            overflow = TextOverflow.Clip,
        )
    }
}

@Composable
private fun NavigationDrawerTvHeader(
    modifier: Modifier = Modifier,
    isExpanded: Boolean,
    daysLeftUntilExpiry: Long?,
    deviceName: String?,
) {
    Column(
        modifier =
            modifier.width(
                if (isExpanded) NavigationDrawerItemDefaults.ExpandedDrawerItemWidth
                else NavigationDrawerItemDefaults.CollapsedDrawerItemWidth
            )
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.mullvadLogoTextStartPadding),
        ) {
            Icon(
                modifier = Modifier.size(Dimens.mediumIconSize),
                painter = painterResource(id = R.drawable.logo_icon),
                contentDescription = null, // No meaningful user info or action.
                tint = Color.Unspecified, // Logo should not be tinted
            )
            if (isExpanded) {
                Icon(
                    modifier = Modifier.height(Dimens.mullvadLogoTextHeight),
                    painter = painterResource(id = R.drawable.logo_text),
                    contentDescription = null, // No meaningful user info or action.
                    tint = Color.Unspecified, // Logo should not be tinted
                )
            }
        }
        Spacer(Modifier.height(8.dp))

        if (isExpanded) {
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(R.string.top_bar_device_name, deviceName ?: ""),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimary,
                maxLines = 1,
                overflow = TextOverflow.Clip,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text =
                    stringResource(
                        id = R.string.top_bar_time_left,
                        pluralStringResource(
                            id = R.plurals.days,
                            daysLeftUntilExpiry?.toInt() ?: 0,
                            daysLeftUntilExpiry ?: 0,
                        ),
                    ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimary,
                maxLines = 1,
                overflow = TextOverflow.Clip,
            )
        }
    }
}
