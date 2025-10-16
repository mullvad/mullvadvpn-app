package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.Language
import androidx.compose.material.icons.filled.LocationOn
import androidx.compose.material.icons.filled.PhoneAndroid
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.layoutId
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight.Companion.SemiBold
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.Dimension.Companion.fillToConstraints
import androidx.constraintlayout.compose.ExperimentalMotionApi
import androidx.constraintlayout.compose.MotionLayout
import androidx.constraintlayout.compose.MotionLayoutScope
import androidx.constraintlayout.compose.MotionScene
import net.mullvad.mullvadvpn.lib.theme.AppTheme

enum class RelayList {
    Entry,
    Exit,
}

@Preview
@Composable
fun ComposableTest() {
    var isMultihop by remember { mutableStateOf(false) }
    var progress by remember { mutableStateOf(0f) }
    var selected by remember { mutableStateOf(RelayList.Exit) }

    AppTheme {
        Surface {
            Column {
                Slider(
                    modifier = Modifier.padding(16.dp),
                    value = progress,
                    onValueChange = { progress = it },
                )
                Switch(isMultihop, onCheckedChange = { isMultihop = it })
                Text("Progress: $progress")
                AnimatedContent(!isMultihop) {
                    if (it) {
                        SingleHopSelector(progress, exitHopState = HopState("Sweden", 0, null))
                    } else {
                        MultihopSelecter(
                            selected,
                            onSelectHop = { selected = it },
                            progress,
                            exitHopState = HopState("Sweden", 0, null),
                            entryHopState = HopState("Sweden", 0, null),
                        )
                    }
                }
            }
        }
    }
}

data class HopState(val text: String, val filters: Int, val errorText: String?)

private val internetHint = "internetHint"
private val internetExitDash = "internetExitDash"
private val exitCenterGuide = "exitCenterGuide"
private val exit = "exit"
private val exitDeviceDash = "exitDeviceDash"
private val deviceHint = "deviceHint"

@OptIn(ExperimentalMotionApi::class)
@Composable
fun SingleHopSelector(progress: Float, exitHopState: HopState, modifier: Modifier = Modifier) {

    val scene = MotionScene {
        val (internetHint, internetExitDash, exit, exitDeviceDash, exitCenterGuide, deviceHint) =
            createRefsFor(
                internetHint,
                internetExitDash,
                exit,
                exitDeviceDash,
                exitCenterGuide,
                deviceHint,
            )

        val expanded =
            constraintSet("expanded") {
                constrain(internetHint) {
                    centerHorizontallyTo(parent)
                    top.linkTo(parent.top)
                }

                constrain(exit) {
                    top.linkTo(internetHint.bottom)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(deviceHint) {
                    linkTo(
                        top = exit.bottom,
                        start = parent.start,
                        end = parent.end,
                        bottom = parent.bottom,
                    )
                }

                constrain(exitCenterGuide) { centerVerticallyTo(exit) }

                val dashGuide = createGuidelineFromStart(21.dp)
                constrain(internetExitDash) {
                    height = fillToConstraints
                    centerAround(dashGuide)
                    linkTo(top = internetHint.bottom, bottom = exitCenterGuide.top)
                }
                constrain(exitDeviceDash) {
                    height = fillToConstraints
                    centerAround(dashGuide)
                    linkTo(top = exitCenterGuide.bottom, bottom = deviceHint.top)
                }
            }

        val collapsed =
            constraintSet("collapsed") {
                constrain(exit) { centerTo(parent) }
                constrain(internetHint, deviceHint, internetExitDash, exitDeviceDash) {
                    centerVerticallyTo(parent)
                }
                val dashGuide = createGuidelineFromStart(21.dp)
                constrain(internetExitDash, exitDeviceDash) { centerAround(dashGuide) }
            }

        defaultTransition(expanded, collapsed) {
            keyAttributes(internetHint, deviceHint) {
                frame(0) { alpha = 1f }
                frame(66) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
            keyAttributes(internetExitDash, exitDeviceDash) {
                frame(0) { alpha = 1f }
                frame(10) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
        }
    }

    CompositionLocalProvider(LocalContentColor provides deselectedColor) {
        MotionLayout(modifier = modifier.fillMaxWidth(), motionScene = scene, progress = progress) {
            LocationHint("Internet", Icons.Default.Language, Modifier.layoutId(internetHint))

            Spacer(modifier = Modifier.size(20.dp).layoutId(exitCenterGuide))

            Icon(
                modifier = Modifier.padding(2.dp).size(14.dp).layoutId(deviceHint),
                imageVector = Icons.Default.PhoneAndroid,
                contentDescription = null,
            )
            LocationHint("Device", Icons.Default.PhoneAndroid, Modifier.layoutId(deviceHint))

            Hop(
                Modifier.layoutId(exit).padding(4.dp),
                hopState = exitHopState,
                Icons.Default.LocationOn,
                selected = true,
                onSelect = {},
                onFilterClick = {},
            )

            DashedLine(modifier = Modifier.layoutId(internetExitDash))
            DashedLine(modifier = Modifier.layoutId(exitDeviceDash))
        }
    }
}

private val panel = "panel"
private val entryCenterGuide = "entryCenterGuide"
private val entry = "entry"
private val exitEntryDash = "exitEntryDash"
private val entryDeviceDash = "entryDeviceDash"

@OptIn(ExperimentalMotionApi::class)
@Composable
fun MultihopSelecter(
    selected: RelayList,
    onSelectHop: (RelayList) -> Unit,
    progress: Float,
    exitHopState: HopState,
    entryHopState: HopState,
    modifier: Modifier = Modifier,
) {
    val scene = MotionScene {
        val (
            internetHint,
            internetExitDash,
            exit,
            exitEntryDash,
            exitCenterGuide,
            entry,
            entryDeviceDash,
            deviceHint,
        ) = createRefsFor(
            internetHint,
            internetExitDash,
            exit,
            exitEntryDash,
            exitCenterGuide,
            entry,
            entryDeviceDash,
            deviceHint,
        )

        val (panel, entryCenterGuide) = createRefsFor(panel, entryCenterGuide)

        val expanded =
            constraintSet("expanded") {
                constrain(internetHint) {
                    linkTo(
                        top = parent.top,
                        start = parent.start,
                        end = parent.end,
                        bottom = exit.top,
                        startMargin = 12.dp,
                    )
                }

                constrain(panel) {
                    width = fillToConstraints
                    height = fillToConstraints
                    linkTo(
                        top = exit.top,
                        start = exit.start,
                        end = exit.end,
                        bottom = entry.bottom,
                    )
                }
                constrain(exit) {
                    top.linkTo(internetHint.bottom)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(exitCenterGuide) { centerVerticallyTo(exit) }

                constrain(entry) {
                    top.linkTo(exit.bottom)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(entryCenterGuide) { centerVerticallyTo(entry) }

                constrain(deviceHint) {
                    linkTo(
                        top = entry.bottom,
                        start = parent.start,
                        end = parent.end,
                        bottom = parent.bottom,
                        startMargin = 12.dp,
                    )
                }

                val dashGuide = createGuidelineFromStart(21.dp)
                constrain(internetExitDash) {
                    height = fillToConstraints
                    centerAround(dashGuide)
                    linkTo(top = internetHint.bottom, bottom = exitCenterGuide.top)
                }
                constrain(exitEntryDash) {
                    height = fillToConstraints
                    centerAround(dashGuide)
                    linkTo(top = exitCenterGuide.bottom, bottom = entryCenterGuide.top)
                }
                constrain(entryDeviceDash) {
                    height = fillToConstraints
                    centerAround(dashGuide)
                    linkTo(top = entryCenterGuide.bottom, bottom = deviceHint.top)
                }
            }

        val collapsed =
            constraintSet("collapsed") {
                constrain(internetHint) {
                    linkTo(parent.start, parent.end)
                    top.linkTo(parent.top)
                }

                createVerticalChain(exit, entry)
                constrain(panel) {
                    width = fillToConstraints
                    height = fillToConstraints
                    linkTo(
                        top = exit.top,
                        start = exit.start,
                        end = exit.end,
                        bottom = entry.bottom,
                    )
                }

                constrain(deviceHint) {
                    centerHorizontallyTo(parent)
                    bottom.linkTo(parent.bottom)
                }

                constrain(exitCenterGuide) { centerVerticallyTo(exit) }
                constrain(entryCenterGuide) { centerVerticallyTo(entry) }

                val dashGuide = createGuidelineFromStart(21.dp)
                constrain(internetExitDash) {
                    centerAround(dashGuide)
                    linkTo(top = parent.top, bottom = exitCenterGuide.top)
                }
                constrain(exitEntryDash) {
                    centerAround(dashGuide)
                    linkTo(top = exitCenterGuide.bottom, bottom = entryCenterGuide.top)
                }

                constrain(entryDeviceDash) {
                    centerAround(dashGuide)
                    linkTo(top = entryCenterGuide.bottom, bottom = parent.bottom)
                }
            }

        defaultTransition(expanded, collapsed) {
            keyAttributes(internetHint, deviceHint) {
                frame(0) { alpha = 1f }
                frame(66) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
            keyAttributes(internetExitDash, exitEntryDash, entryDeviceDash) {
                frame(0) { alpha = 1f }
                frame(20) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
        }
    }

    CompositionLocalProvider(LocalContentColor provides deselectedColor) {
        MotionLayout(
            modifier = modifier.fillMaxWidth(),
            motionScene = scene,
            progress = progress,
            //            debugFlags = DebugFlags(true, true, true),
        ) {
            LocationHint("Internet", Icons.Default.Language, Modifier.layoutId(internetHint))
            LocationHint("Your device", Icons.Default.PhoneAndroid, Modifier.layoutId(internetHint))

            Box(
                Modifier.layoutId(panel)
                    .clip(RoundedCornerShape(16.dp))
                    .background(Color(0xFF101823))
            ) {}
            Spacer(modifier = Modifier.size(20.dp).layoutId(exitCenterGuide))
            Hop(
                Modifier.layoutId(exit).padding(4.dp),
                hopState = exitHopState,
                Icons.Default.LocationOn,
                selected = selected == RelayList.Exit,
                onSelect = { onSelectHop(RelayList.Exit) },
                onFilterClick = {},
            )

            Spacer(modifier = Modifier.size(20.dp).layoutId(entryCenterGuide))
            Hop(
                Modifier.layoutId(entry).padding(4.dp),
                hopState = entryHopState,
                Icons.Default.Storage,
                selected = selected == RelayList.Entry,
                onSelect = { onSelectHop(RelayList.Entry) },
                onFilterClick = {},
            )

            DashedLine(modifier = Modifier.layoutId(internetExitDash))
            DashedLine(modifier = Modifier.layoutId(exitEntryDash))
            DashedLine(modifier = Modifier.layoutId(entryDeviceDash))
        }
    }
}

private val deselectedColor = Color(0xFFA3ABB5)

@Composable
private fun DashedLine(modifier: Modifier) {
    Canvas(modifier.width(2.dp)) {
        val x = size.width / 2
        val lineLength = 2.dp.toPx()
        val gapLength = 5.dp.toPx() // 2.dp will be taken by StrokeCap.Round
        val capRadius = size.width / 2

        val interval = floatArrayOf(lineLength, gapLength)

        val period = interval.sum()
        val visualGap = gapLength - 2 * capRadius
        val remainder = (size.height + visualGap) % period

        val pathEffect = PathEffect.dashPathEffect(interval)

        drawLine(
            deselectedColor,
            start = Offset(x, capRadius + remainder / 2),
            end = Offset(x, size.height),
            strokeWidth = size.width,
            cap = StrokeCap.Round,
            pathEffect = pathEffect,
        )
    }
}

@Composable
private fun LocationHint(text: String, imageVector: ImageVector, modifier: Modifier = Modifier) {
    Row(
        modifier.padding(start = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Icon(
            modifier = Modifier.padding(2.dp).size(14.dp),
            imageVector = imageVector,
            contentDescription = null,
        )
        Text(modifier = Modifier.weight(1f), text = text)
    }
}

@OptIn(ExperimentalMotionApi::class)
@Composable
private fun Hop(
    modifier: Modifier,
    hopState: HopState,
    leadingIcon: ImageVector,
    selected: Boolean,
    onSelect: () -> Unit,
    onFilterClick: () -> Unit,
) {
    CompositionLocalProvider(
        LocalContentColor provides
            if (selected) MaterialTheme.colorScheme.onPrimary else deselectedColor
    ) {
        val alpha by animateFloatAsState(if (selected) 1f else 0f, tween())

        Column {
            Row(
                modifier =
                    modifier
                        .semantics {
                            role = Role.Switch
                            this.selected = selected
                        }
                        .clip(RoundedCornerShape(12.dp))
                        .clickable(onClick = onSelect)
                        .background(MaterialTheme.colorScheme.primary.copy(alpha = alpha)),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    modifier =
                        Modifier.padding(start = 8.dp, end = 8.dp - 4.dp * alpha).size(18.dp),
                    imageVector = leadingIcon,
                    contentDescription = null,
                )
                Text(
                    modifier = Modifier.weight(1f),
                    text = hopState.text,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = SemiBold,
                )
                FilterButton(onClick = {}, filters = hopState.filters)
            }
            if (hopState.errorText != null) {
                Text(
                    modifier = Modifier.padding(start = 30.dp),
                    text = hopState.errorText,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

@Preview
@Composable
fun DashedLinePreview() {
    var progress by remember { mutableFloatStateOf(0f) }
    AppTheme {
        Surface {
            Column {
                Slider(progress, { progress = it }, Modifier.padding(16.dp))
                Box(modifier = Modifier.size(20.dp).background(Color.Red))
                DashedLine(Modifier.size(height = 100.dp * progress, width = 2.dp))
                Box(modifier = Modifier.size(20.dp).background(Color.Red))
            }
        }
    }
}

@Preview
@Composable
fun FilterButtonPreview() {
    AppTheme {
        Column {
            FilterButton()
            FilterButton(3)
            FilterButton(10)
        }
    }
}

@Composable
fun FilterButton(filters: Int = 0, onClick: () -> Unit = {}) {
    IconButton(modifier = Modifier.drawWithContent { drawContent() }, onClick = onClick) {
        BadgedBox(
            badge = {
                if (filters > 0) {
                    Badge(containerColor = Color.Transparent) { Text(filters.toString()) }
                }
            }
        ) {
            Icon(imageVector = Icons.Default.FilterList, contentDescription = null)
        }
    }
}
