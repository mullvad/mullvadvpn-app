package net.mullvad.mullvadvpn.feature.home.impl.connect

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.PauseVpnNavResult
import net.mullvad.mullvadvpn.lib.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.lib.ui.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.ui.component.listitem.BottomSheetListItem
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.PAUSE_VPN_DURATION_X_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

private val PAUSE_DURATIONS = listOf(5.minutes, 15.minutes, 30.minutes, 1.hours)

@Preview
@Composable
private fun PreviewPauseVpnBottomSheetContent() {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)) {
            PauseVpnBottomSheetContent(
                backgroundColor = MaterialTheme.colorScheme.surfaceContainer,
                onBackgroundColor = MaterialTheme.colorScheme.onSurface,
                onDurationClick = {},
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PauseVpnBottomSheet(navigator: Navigator) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    val backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface

    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { navigator.goBack() },
    ) {
        PauseVpnBottomSheetContent(
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onDurationClick =
                dropUnlessResumed { duration: Duration ->
                    navigator.goBack(result = PauseVpnNavResult(seconds = duration.inWholeSeconds))
                },
        )
    }
}

@Composable
private fun PauseVpnBottomSheetContent(
    backgroundColor: Color,
    onBackgroundColor: Color,
    onDurationClick: (Duration) -> Unit,
) {
    BottomSheetListItem(
        title = stringResource(id = R.string.pause_vpn),
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
    )
    HorizontalDivider(color = onBackgroundColor)
    PAUSE_DURATIONS.forEach { duration ->
        BottomSheetListItem(
            title = duration.toText(),
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            modifier =
                @Suppress("ImplicitDefaultLocale")
                Modifier.testTag(PAUSE_VPN_DURATION_X_TEST_TAG.format(duration.inWholeMinutes)),
            onClick = { onDurationClick(duration) },
        )
    }
    HorizontalDivider(color = onBackgroundColor)
    Row(verticalAlignment = Alignment.CenterVertically) {
        Icon(
            modifier = Modifier.padding(Dimens.mediumPadding),
            imageVector = Icons.Rounded.Info,
            tint = MaterialTheme.colorScheme.error,
            contentDescription = null,
        )
        Text(
            modifier =
                Modifier.padding(
                    top = Dimens.smallPadding,
                    end = Dimens.mediumPadding,
                    bottom = Dimens.smallPadding,
                ),
            text = stringResource(R.string.pause_vpn_warning),
            style = MaterialTheme.typography.labelLarge,
            color = onBackgroundColor,
        )
    }
}

@Composable
private fun Duration.toText(): String =
    if (this < 1.hours) {
        val minutes = inWholeMinutes.toInt()
        pluralStringResource(id = R.plurals.pause_vpn_minutes, count = minutes, minutes)
    } else {
        val hours = inWholeHours.toInt()
        pluralStringResource(id = R.plurals.pause_vpn_hours, count = hours, hours)
    }
