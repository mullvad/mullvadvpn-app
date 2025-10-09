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
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
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
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension.Companion.fillToConstraints
import androidx.constraintlayout.compose.Dimension.Companion.preferredWrapContent
import androidx.constraintlayout.compose.ExperimentalMotionApi
import androidx.constraintlayout.compose.MotionLayout
import androidx.constraintlayout.compose.MotionScene
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
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
                        MultiHopSelecter(selected, onSelect = { selected = it }, progress)
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
                "Exit Location",
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
fun MultiHopSelecter(
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
                "Exit Location",
                Icons.Default.LocationOn,
                selected = selected,
                onSelect = { onSelect(true) },
            )

            Spacer(modifier = Modifier.size(20.dp).layoutId(entryCenterGuide))
            Hop(
                Modifier.layoutId(entry).padding(4.dp),
                "Entry Location",
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

@Preview
@Composable
fun HopSelectorPreview() {
    var selectedList by remember { mutableStateOf(RelayList.Entry) }
    var isMultihop by remember { mutableStateOf(true) }
    val entry =
        RelayItem.Location.Country(
            id = GeoLocationId.Country("se"),
            name = "Sweden",
            cities = listOf(),
        )
    val exit =
        RelayItem.Location.Country(
            id = GeoLocationId.Country("de"),
            name = "Germany",
            cities = listOf(),
        )

    AppTheme {
        Surface {
            Column(modifier = Modifier.fillMaxSize()) {
                Switch(isMultihop, onCheckedChange = { isMultihop = it })
                Spacer(Modifier.size(32.dp))

                HopSelector(
                    modifier = Modifier.padding(8.dp),
                    if (isMultihop) Hop.Multi(entry, exit) else Hop.Single(exit),
                    selectedList = if (isMultihop) selectedList else RelayList.Exit,
                    onSelect = { selectedList = it },
                )
            }
        }
    }
}

private val deselectedColor = Color(0xFFA3ABB5)

@Composable
fun HopSelector(
    modifier: Modifier,
    hop: Hop,
    selectedList: RelayList,
    onSelect: (RelayList) -> Unit,
) {
    CompositionLocalProvider(LocalContentColor provides deselectedColor) {
        Column {
            ConstraintLayout(modifier = modifier, animateChangesSpec = tween()) {
                val (internetIcon, internetText) = createRefs()
                val internetBottomBarrier = createBottomBarrier(internetIcon, internetText)
                val (dashInternetExit, dashExitEntry, dashEntryDevice) = createRefs()
                val hopBox = createRef()
                val (hopEntry, hopExit) = createRefs()
                val (deviceIcon, deviceText) = createRefs()

                val startGuide = createGuidelineFromStart(12.dp)

                Icon(
                    modifier =
                        Modifier.constrainAs(internetIcon) {
                                start.linkTo(startGuide)
                                top.linkTo(parent.top)
                                bottom.linkTo(hopExit.top, margin = 4.dp)
                            }
                            .size(18.dp),
                    imageVector = Icons.Default.Language,
                    contentDescription = null,
                )
                Text(
                    modifier =
                        Modifier.constrainAs(internetText) {
                            width = fillToConstraints
                            start.linkTo(internetIcon.end, margin = 10.dp)
                            end.linkTo(parent.end)
                            top.linkTo(parent.top)
                            bottom.linkTo(hopExit.top, margin = 4.dp)
                        },
                    text = "Internet",
                    style = MaterialTheme.typography.bodyMedium,
                )

                val alpha by animateFloatAsState(if (hop is Hop.Multi) 1f else 0f, tween(500))
                Box(
                    modifier =
                        Modifier.constrainAs(hopBox) {
                                width = fillToConstraints
                                height = fillToConstraints
                                top.linkTo(hopExit.top)
                                if (hop is Hop.Multi) {
                                    bottom.linkTo(hopEntry.bottom)
                                } else {
                                    bottom.linkTo(hopExit.bottom)
                                }
                                start.linkTo(parent.start)
                                end.linkTo(parent.end)
                            }
                            .clip(RoundedCornerShape(16.dp))
                            .background(Color(0xFF101823).copy(alpha = alpha))
                ) {}

                Hop(
                    Modifier.constrainAs(hopExit) {
                            top.linkTo(internetBottomBarrier, margin = 8.dp)
                            start.linkTo(parent.start)
                            end.linkTo(parent.end)
                            bottom.linkTo(hopExit.top)
                        }
                        .padding(4.dp),
                    hop.exit().name,
                    Icons.Default.LocationOn,
                    selected = selectedList == RelayList.Exit,
                    onSelect = { onSelect(RelayList.Exit) },
                )

                var entryLocation by remember {
                    val locationName = if (hop is Hop.Multi) hop.entry.name else null
                    mutableStateOf(locationName)
                }

                if (hop is Hop.Multi && hop.entry.name != entryLocation) {
                    entryLocation = hop.entry.name
                }

                val entryAlpha by
                    animateFloatAsState(
                        if (hop is Hop.Multi) 1f else 0f,
                        tween(delayMillis = 100, durationMillis = 100),
                    )
                Hop(
                    modifier =
                        Modifier.constrainAs(hopEntry) {
                                height = preferredWrapContent
                                width = fillToConstraints
                                // Hack to fix size of the view
                                if (hop is Hop.Multi) {
                                    top.linkTo(hopExit.bottom)
                                } else {
                                    top.linkTo(hopExit.top)
                                }
                                start.linkTo(parent.start)
                                end.linkTo(parent.end)
                            }
                            .alpha(entryAlpha)
                            .padding(4.dp),
                    entryLocation ?: "",
                    Icons.Default.Storage,
                    selected = selectedList == RelayList.Entry,
                    onSelect = { onSelect(RelayList.Entry) },
                )

                Icon(
                    modifier =
                        Modifier.constrainAs(deviceIcon) {
                                if (hop is Hop.Multi) {
                                    top.linkTo(hopEntry.bottom, margin = 4.dp)
                                } else {
                                    top.linkTo(hopExit.bottom, margin = 4.dp)
                                }
                                bottom.linkTo(parent.bottom)
                                start.linkTo(startGuide)
                            }
                            .size(18.dp),
                    imageVector = Icons.Default.PhoneAndroid,
                    contentDescription = null,
                )
                Text(
                    modifier =
                        Modifier.constrainAs(deviceText) {
                            width = fillToConstraints
                            if (hop is Hop.Multi) {
                                top.linkTo(hopEntry.bottom, margin = 4.dp)
                            } else {
                                top.linkTo(hopExit.bottom)
                            }
                            bottom.linkTo(parent.bottom)
                            start.linkTo(deviceIcon.end, margin = 10.dp)
                            end.linkTo(parent.end)
                        },
                    text = "Device",
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}

@Composable
private fun DashedLine(modifier: Modifier) {
    val pathEffect = PathEffect.dashPathEffect(floatArrayOf(10f, 12f), 0f)
    Canvas(modifier.width(2.dp)) {
        val x = size.width / 2
        drawLine(
            deselectedColor,
            start = Offset(x, 0f),
            end = Offset(size.width / 2, size.height),
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
        Row(
            modifier =
                modifier
                    .semantics {
                        role = Role.Switch
                        this.selected = selected
                    }
                    .clip(RoundedCornerShape(12.dp))
                    .clickable(onClick = onSelect)
                    .let { if (selected) it.background(MaterialTheme.colorScheme.primary) else it },
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier = Modifier.padding(start = 8.dp, end = 4.dp).size(18.dp),
                imageVector = imageVector,
                contentDescription = null,
            )
            Text(modifier = Modifier.weight(1f), text = text)
            IconButton(onClick = {}) { Icon(Icons.Default.FilterList, contentDescription = null) }
        }
    }
}
