package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.constraintlayout.compose.MotionScene
import net.mullvad.mullvadvpn.lib.theme.AppTheme

enum class RelayList {
    Entry,
    Exit,
}

@Preview
@Composable
fun ComposableTest() {
    var isMultihop by remember { mutableStateOf(true) }
    var progress by remember { mutableStateOf(0f) }
    var selected by remember { mutableStateOf(true) }

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
                AnimatedContent(isMultihop) {
                    if (it) {
                        SingleHopSelector(progress)
                    } else {
                        MultihopSelecter(selected, onSelect = { selected = it }, progress)
                    }
                }
            }
        }
    }
}

private val internetIcon = "internetIcon"
private val internetText = "internetText"
private val internetExitDash = "internetExitDash"
private val exitCenterGuide = "exitCenterGuide"
private val exit = "exit"
private val exitDeviceDash = "exitDeviceDash"
private val deviceIcon = "deviceIcon"
private val deviceText = "deviceText"

@OptIn(ExperimentalMotionApi::class)
@Composable
fun SingleHopSelector(progress: Float, modifier: Modifier = Modifier) {

    val scene = MotionScene {
        val (
            internetIcon,
            internetText,
            internetExitDash,
            exit,
            exitDeviceDash,
            exitCenterGuide,
            deviceIcon,
            deviceText) =
            createRefsFor(
                internetIcon,
                internetText,
                internetExitDash,
                exit,
                exitDeviceDash,
                exitCenterGuide,
                deviceIcon,
                deviceText,
            )

        val expanded =
            constraintSet("expanded") {
                constrain(internetIcon) {
                    linkTo(
                        top = parent.top,
                        start = parent.start,
                        end = internetText.start,
                        bottom = exit.top,
                        startMargin = 12.dp,
                    )
                }
                constrain(internetText) {
                    centerVerticallyTo(internetIcon)
                    width = fillToConstraints
                    start.linkTo(internetIcon.end, 8.dp)
                    end.linkTo(parent.end)
                }
                val internetBottomBarrier = createBottomBarrier(internetIcon, internetText)
                constrain(exit) {
                    top.linkTo(internetBottomBarrier)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(deviceIcon) {
                    linkTo(
                        top = exit.bottom,
                        start = parent.start,
                        end = deviceText.start,
                        bottom = parent.bottom,
                        startMargin = 12.dp,
                    )
                }
                constrain(deviceText) {
                    centerVerticallyTo(deviceIcon)
                    width = fillToConstraints
                    start.linkTo(deviceIcon.end, 8.dp)
                    end.linkTo(parent.end)
                }

                constrain(exitCenterGuide) { centerVerticallyTo(exit) }

                constrain(internetExitDash) {
                    height = fillToConstraints
                    linkTo(top = internetIcon.bottom, bottom = exitCenterGuide.top)
                    centerHorizontallyTo(internetIcon)
                }
                constrain(exitDeviceDash) {
                    height = fillToConstraints
                    linkTo(top = exitCenterGuide.bottom, bottom = deviceIcon.top)
                    centerHorizontallyTo(deviceIcon)
                }
            }

        val collapsed =
            constraintSet("collapsed", expanded) {
                constrain(exit) {
                    centerVerticallyTo(parent)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(
                    internetIcon,
                    internetText,
                    deviceIcon,
                    deviceText,
                    internetExitDash,
                    exitDeviceDash,
                ) {
                    centerVerticallyTo(parent)
                }
            }

        defaultTransition(expanded, collapsed) {
            keyAttributes(internetIcon, internetText, deviceIcon, deviceText) {
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
            Icon(
                modifier = Modifier.padding(2.dp).size(14.dp).layoutId(internetIcon),
                imageVector = Icons.Default.Language,
                contentDescription = null,
            )
            Text(modifier = Modifier.layoutId(internetText), text = "Internet")

            Spacer(modifier = Modifier.size(20.dp).layoutId(exitCenterGuide))

            Icon(
                modifier = Modifier.padding(2.dp).size(14.dp).layoutId(deviceIcon),
                imageVector = Icons.Default.PhoneAndroid,
                contentDescription = null,
            )
            Text(modifier = Modifier.layoutId(deviceText), text = "Your device")

            Hop(
                Modifier.layoutId(exit).padding(4.dp),
                "Sweden",
                Icons.Default.LocationOn,
                selected = true,
                onSelect = {},
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
    selected: Boolean,
    onSelect: (Boolean) -> Unit,
    progress: Float,
    modifier: Modifier = Modifier,
) {
    val scene = MotionScene {
        val (
            internetIcon,
            internetText,
            internetExitDash,
            exit,
            exitEntryDash,
            exitCenterGuide,
            entry,
            entryDeviceDash,
            deviceIcon,
            deviceText) =
            createRefsFor(
                internetIcon,
                internetText,
                internetExitDash,
                exit,
                exitEntryDash,
                exitCenterGuide,
                entry,
                entryDeviceDash,
                deviceIcon,
                deviceText,
            )

        val (panel, entryCenterGuide) = createRefsFor(panel, entryCenterGuide)

        val expanded =
            constraintSet("expanded") {
                val internetBottomBarrier = createBottomBarrier(internetIcon, internetText)
                constrain(internetIcon) {
                    linkTo(
                        top = parent.top,
                        start = parent.start,
                        end = internetText.start,
                        bottom = internetBottomBarrier,
                        startMargin = 12.dp,
                    )
                }
                constrain(internetText) {
                    centerVerticallyTo(internetIcon)
                    width = fillToConstraints
                    start.linkTo(internetIcon.end, 8.dp)
                    end.linkTo(parent.end)
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
                    top.linkTo(internetBottomBarrier)
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

                val deviceBarrierBottom = createBottomBarrier(deviceIcon, deviceText)
                constrain(deviceIcon) {
                    linkTo(
                        top = entry.bottom,
                        start = parent.start,
                        end = deviceText.start,
                        bottom = deviceBarrierBottom,
                        startMargin = 12.dp,
                    )
                }
                constrain(deviceText) {
                    centerVerticallyTo(deviceIcon)
                    width = fillToConstraints
                    start.linkTo(deviceIcon.end, 8.dp)
                    end.linkTo(parent.end)
                }

                constrain(internetExitDash) {
                    height = fillToConstraints
                    linkTo(top = internetIcon.bottom, bottom = exitCenterGuide.top)
                    centerHorizontallyTo(internetIcon)
                }
                constrain(exitEntryDash) {
                    height = fillToConstraints
                    linkTo(top = exitCenterGuide.bottom, bottom = entryCenterGuide.top)
                    centerHorizontallyTo(deviceIcon)
                }
                constrain(entryDeviceDash) {
                    height = fillToConstraints
                    linkTo(top = entryCenterGuide.bottom, bottom = deviceIcon.top)
                    centerHorizontallyTo(deviceIcon)
                }
            }

        val collapsed =
            constraintSet("collapsed", expanded) {
                constrain(exit) {
                    top.linkTo(parent.top)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(entry) {
                    top.linkTo(exit.bottom)
                    start.linkTo(parent.start)
                    end.linkTo(parent.end)
                }
                constrain(
                    deviceIcon,
                    deviceText,
                    internetExitDash,
                    exitEntryDash,
                    entryDeviceDash,
                ) {
                    centerVerticallyTo(parent)
                }
            }

        defaultTransition(expanded, collapsed) {
            keyAttributes(internetIcon, internetText, deviceIcon, deviceText) {
                frame(0) { alpha = 1f }
                frame(66) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
            keyAttributes(internetExitDash, exitEntryDash, entryDeviceDash) {
                frame(0) { alpha = 1f }
                frame(10) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
        }
    }

    CompositionLocalProvider(LocalContentColor provides deselectedColor) {
        MotionLayout(modifier = modifier.fillMaxWidth(), motionScene = scene, progress = progress) {
            Icon(
                modifier = Modifier.padding(2.dp).size(14.dp).layoutId(internetIcon),
                imageVector = Icons.Default.Language,
                contentDescription = null,
            )
            Text(modifier = Modifier.layoutId(internetText), text = "Internet")

            Icon(
                modifier = Modifier.padding(2.dp).size(14.dp).layoutId(deviceIcon),
                imageVector = Icons.Default.PhoneAndroid,
                contentDescription = null,
            )
            Text(modifier = Modifier.layoutId(deviceText), text = "Your device")

            Box(
                Modifier.layoutId(panel)
                    .clip(RoundedCornerShape(16.dp))
                    .background(Color(0xFF101823))
            ) {}
            Spacer(modifier = Modifier.size(20.dp).layoutId(exitCenterGuide))
            Hop(
                Modifier.layoutId(exit).padding(4.dp),
                "Germany",
                Icons.Default.LocationOn,
                selected = selected,
                onSelect = { onSelect(true) },
            )

            Spacer(modifier = Modifier.size(20.dp).layoutId(entryCenterGuide))
            Hop(
                Modifier.layoutId(entry).padding(4.dp),
                "Sweden",
                Icons.Default.Storage,
                selected = !selected,
                onSelect = { onSelect(false) },
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
private fun Hop(
    modifier: Modifier,
    text: String,
    imageVector: ImageVector,
    selected: Boolean,
    onSelect: () -> Unit,
) {
    CompositionLocalProvider(
        LocalContentColor provides
            if (selected) MaterialTheme.colorScheme.onPrimary else deselectedColor
    ) {
        val alpha by animateFloatAsState(if (selected) 1f else 0f, tween())

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
                modifier = Modifier.padding(start = 8.dp, end = 8.dp - 4.dp * alpha).size(18.dp),
                imageVector = imageVector,
                contentDescription = null,
            )
            Text(
                modifier = Modifier.weight(1f),
                text = text,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = SemiBold,
            )
            IconButton(onClick = {}) { Icon(Icons.Default.FilterList, contentDescription = null) }
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
