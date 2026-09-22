package net.mullvad.mullvadvpn.app

import androidx.compose.runtime.Composable
import com.composea11yscanner.triggers.scanOnShake
import kotlin.time.Duration.Companion.seconds

private val SHAKE_TRIGGER_INTERVAL = 2.seconds

@Composable
fun OptionalTools() {
    scanOnShake(minTriggerIntervalMillis = SHAKE_TRIGGER_INTERVAL.inWholeMilliseconds)
}
