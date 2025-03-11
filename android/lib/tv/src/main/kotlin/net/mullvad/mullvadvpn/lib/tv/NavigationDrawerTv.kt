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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
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
import androidx.tv.material3.rememberDrawerState
import net.mullvad.mullvadvpn.lib.theme.AppTheme

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
    if (drawerState.currentValue == DrawerValue.Open) {
        BackHandler(onBack = { drawerState.setValue(DrawerValue.Closed) })
    }
    val brush = Brush.horizontalGradient(listOf(Color.Black, Color.Transparent))

    ModalNavigationDrawer(
        drawerState = drawerState,
        scrimBrush = brush,
        drawerContent = {
            val animatedPadding = animateDpAsState(if (hasFocus) 20.dp else 16.dp)
            Box(
                Modifier.fillMaxHeight()
                    .background(brush)
                    .padding(top = 24.dp, bottom = 24.dp, start = 12.dp, end = 12.dp)
                    .selectableGroup()
            ) {
                NavigationDrawerTvHeader(
                    modifier =
                        Modifier.align(Alignment.TopStart).padding(start = animatedPadding.value),
                    isExpanded = hasFocus,
                    daysLeftUntilExpiry = daysLeftUntilExpiry,
                    deviceName = deviceName,
                )

                NavigationDrawerItem(
                    modifier = Modifier.align(Alignment.CenterStart),
                    onClick = onAccountClick,
                    selected = false,
                    leadingContent = {
                        Icon(
                            tint = MaterialTheme.colorScheme.onPrimary,
                            imageVector = Icons.Default.AccountCircle,
                            contentDescription = null,
                        )
                    },
                ) {
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        color = MaterialTheme.colorScheme.onPrimary,
                        text = "Account",
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                    )
                }

                NavigationDrawerItem(
                    modifier = Modifier.align(Alignment.BottomStart),
                    onClick = onSettingsClick,
                    selected = false,
                    leadingContent = {
                        Icon(
                            tint = MaterialTheme.colorScheme.onPrimary,
                            imageVector = Icons.Default.Settings,
                            contentDescription = null,
                        )
                    },
                ) {
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        color = MaterialTheme.colorScheme.onPrimary,
                        text = "Settings",
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                    )
                }
            }
        },
        content = content,
    )
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
            horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Icon(
                modifier = Modifier.size(32.dp),
                painter = painterResource(id = R.drawable.logo_icon),
                contentDescription = null, // No meaningful user info or action.
                tint = Color.Unspecified, // Logo should not be tinted
            )
            if (isExpanded) {
                Icon(
                    modifier = Modifier.height(13.dp),
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
