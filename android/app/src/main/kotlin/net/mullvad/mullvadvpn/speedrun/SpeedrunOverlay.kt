package net.mullvad.mullvadvpn.speedrun

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.delay
import org.koin.compose.koinInject

/**
 * TEMPORARY global speed-run HUD, drawn on top of the whole app (see MullvadApp). Nothing here
 * consumes touch events except the Start button, so the app stays fully usable "through" the
 * overlay while a run is in progress.
 */
@Composable
fun SpeedrunOverlay(modifier: Modifier = Modifier) {
    val controller = koinInject<SpeedrunController>()
    val state by controller.uiState.collectAsStateWithLifecycle()

    when (state.phase) {
        SpeedrunPhase.NOT_STARTED ->
            if (state.baselineKnown && !state.baselineClean) {
                // Known-dirty app: refuse to start until it is reset.
                HudPill(modifier = modifier) {
                    PillText("Clear app data to start a fair run", size = 13.sp)
                }
            } else {
                // Clean, or still waiting for the daemon (e.g. on the privacy/splash screens).
                StartPill(modifier = modifier, onStart = controller::start)
            }
        SpeedrunPhase.RUNNING -> RunningPill(modifier = modifier, state = state)
        SpeedrunPhase.FINISHED -> FinishedPill(modifier = modifier, state = state)
    }
}

@Composable
private fun StartPill(modifier: Modifier, onStart: () -> Unit) {
    HudPill(modifier = modifier) {
        PillText("🏁 Mullvad Speedrun", size = 16.sp, weight = FontWeight.Bold)
        PillText(
            "Welcome! This is the official Mullvad team-fair presentation speedrun. After you " +
                "press Start you'll complete ${SPEEDRUN_TASKS.size} tasks, and the timer stops " +
                "automatically once they're all done.",
            size = 13.sp,
        )
        PillText("Good luck — may the fastest player win our precious Kodee! 🏆", size = 13.sp)
        Button(onClick = onStart) { Text("Start") }
    }
}

@Composable
private fun RunningPill(modifier: Modifier, state: SpeedrunUiState) {
    var nowMillis by remember { mutableLongStateOf(System.currentTimeMillis()) }
    LaunchedEffect(Unit) {
        while (true) {
            nowMillis = System.currentTimeMillis()
            delay(TICK_MILLIS)
        }
    }
    val elapsed = (nowMillis - state.startTimeMillis).coerceAtLeast(0L)

    HudPill(modifier = modifier) {
        PillText(
            "⏱️ ${formatElapsed(elapsed)}",
            size = 22.sp,
            weight = FontWeight.Bold,
            mono = true,
        )
        if (SpeedrunConfig.SHOW_CURRENT_TASK) {
            state.currentTask?.let { task ->
                PillText(
                    "☑️ ${state.currentIndex}/${state.totalTasks}   👉 ${task.title}",
                    size = 12.sp,
                )
            }
        }
    }
}

@Composable
private fun FinishedPill(modifier: Modifier, state: SpeedrunUiState) {
    HudPill(modifier = modifier) {
        PillText(
            "🏁 ${formatElapsed(state.finalElapsedMillis)}",
            size = 22.sp,
            weight = FontWeight.Bold,
            mono = true,
        )
        PillText(
            "Good job! Show this screen to an Android representative to get on the leaderboard 🏆",
            size = 12.sp,
        )
        PillText("Clear app data to run again.", size = 11.sp)
    }
}

/**
 * The card. Only draw/layout modifiers are used (no `clickable`/`pointerInput`), so it never
 * intercepts touches; the Start button is the only interactive child.
 */
@Composable
private fun HudPill(modifier: Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier =
            modifier
                .statusBarsPadding()
                .padding(top = 4.dp)
                .widthIn(max = 320.dp)
                .background(HudBackground, RoundedCornerShape(12.dp))
                .padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
        content = content,
    )
}

@Composable
private fun PillText(
    text: String,
    size: TextUnit,
    weight: FontWeight = FontWeight.Normal,
    mono: Boolean = false,
) {
    Text(
        text = text,
        color = Color.White,
        fontSize = size,
        fontWeight = weight,
        fontFamily = if (mono) FontFamily.Monospace else FontFamily.Default,
        textAlign = TextAlign.Center,
    )
}

private val HudBackground = Color(0xCC101010L)

private const val TICK_MILLIS = 31L
private const val MILLIS_PER_CENTI = 10
private const val CENTIS_PER_SECOND = 100
private const val SECONDS_PER_MINUTE = 60

private fun formatElapsed(millis: Long): String {
    val totalCentis = millis / MILLIS_PER_CENTI
    val centis = totalCentis % CENTIS_PER_SECOND
    val totalSeconds = totalCentis / CENTIS_PER_SECOND
    val seconds = totalSeconds % SECONDS_PER_MINUTE
    val minutes = totalSeconds / SECONDS_PER_MINUTE
    return "%02d:%02d.%02d".format(minutes, seconds, centis)
}
