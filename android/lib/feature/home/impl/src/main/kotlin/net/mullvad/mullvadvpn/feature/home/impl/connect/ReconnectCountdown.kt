package net.mullvad.mullvadvpn.feature.home.impl.connect

import android.text.format.DateUtils
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import java.time.Duration
import java.time.Instant
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryTextButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CANCEL_PAUSE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECONNECT_COUNTDOWN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewReconnectCountdown() {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            ReconnectCountdown(
                resumeAt = Instant.now().plusSeconds(PREVIEW_SECONDS),
                onCancelClick = {},
            )
        }
    }
}

@Composable
fun ReconnectCountdown(
    resumeAt: Instant,
    onCancelClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val secondsLeft by
        produceState(initialValue = secondsUntil(resumeAt), resumeAt) {
            while (value > 0) {
                // Wake up right after the next whole second has passed.
                delay(Duration.between(Instant.now(), resumeAt).toMillis() % MILLIS_PER_SECOND + 1)
                value = secondsUntil(resumeAt)
            }
        }

    Row(modifier = modifier, verticalAlignment = Alignment.CenterVertically) {
        Text(
            modifier = Modifier.weight(1f).testTag(RECONNECT_COUNTDOWN_TEST_TAG),
            text = stringResource(R.string.reconnects_in, DateUtils.formatElapsedTime(secondsLeft)),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        PrimaryTextButton(
            onClick = onCancelClick,
            text = stringResource(R.string.cancel),
            modifier = Modifier.testTag(CANCEL_PAUSE_BUTTON_TEST_TAG),
        )
    }
}

// Rounded up, so that the countdown only shows zero once the time is actually up.
private fun secondsUntil(instant: Instant): Long {
    val millis = Duration.between(Instant.now(), instant).toMillis().coerceAtLeast(0)
    return (millis + MILLIS_PER_SECOND - 1) / MILLIS_PER_SECOND
}

private const val MILLIS_PER_SECOND = 1000L
private const val PREVIEW_SECONDS = 299L
