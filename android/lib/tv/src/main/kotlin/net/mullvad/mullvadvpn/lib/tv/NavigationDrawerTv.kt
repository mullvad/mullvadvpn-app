package net.mullvad.mullvadvpn.lib.tv

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.unit.dp
import androidx.tv.material3.DrawerValue
import androidx.tv.material3.ModalNavigationDrawer
import androidx.tv.material3.NavigationDrawerItem
import androidx.tv.material3.rememberDrawerState

@Composable
@Suppress("LongMethod")
fun NavigationDrawerTv(
    daysLeftUntilExpiry: Long?,
    deviceName: String?,
    onSettingsClick: (() -> Unit),
    onAccountClick: (() -> Unit),
    content: @Composable () -> Unit,
) {
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    if (drawerState.currentValue == DrawerValue.Open) {
        BackHandler(onBack = { drawerState.setValue(DrawerValue.Closed) })
    }
    ModalNavigationDrawer(
        drawerContent = {
            Column(
                Modifier.background(
                        Brush.horizontalGradient(listOf(Color.Black, Color.Transparent))
                    )
                    .fillMaxHeight()
                    .padding(12.dp),
                horizontalAlignment = Alignment.Start,
                verticalArrangement = Arrangement.SpaceBetween,
            ) {
                val animatedPadding = animateDpAsState(if (hasFocus) 4.dp else 0.dp)
                Column(modifier = Modifier.weight(1f)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            painter = painterResource(id = R.drawable.logo_icon),
                            contentDescription = null, // No meaningful user info or action.
                            modifier =
                                Modifier.padding(start = animatedPadding.value)
                                    //                                    .padding(16.dp)
                                    .size(32.dp),
                            tint = Color.Unspecified, // Logo should not be tinted
                        )
                        if (hasFocus) {
                            Icon(
                                modifier = Modifier.height(16.dp),
                                painter = painterResource(id = R.drawable.logo_text),
                                contentDescription = null, // No meaningful user info or action.
                                tint = Color.Unspecified, // Logo should not be tinted
                            )
                        }
                    }

                    if (hasFocus) {
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
                            maxLines = 1,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                        Text(
                            modifier = Modifier.fillMaxWidth(),
                            color = MaterialTheme.colorScheme.onPrimary,
                            text = deviceName ?: "",
                            maxLines = 1,
                            overflow = TextOverflow.Clip,
                        )
                    }
                }

                NavigationDrawerItem(
                    modifier = Modifier.weight(1f),
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
                    modifier = Modifier.weight(1f),
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
        drawerState = drawerState,
        scrimBrush = Brush.horizontalGradient(listOf(Color.Black, Color.Transparent)),
        content = content,
    )
}
