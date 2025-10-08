package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.AnimatedVisibility
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
    var isCollapsed by remember { mutableStateOf(true) }
    var progress by remember { mutableStateOf(0f) }
    Column {
        Slider(
            modifier = Modifier.padding(16.dp),
            value = progress,
            onValueChange = { progress = it },
        )
        SimpleHopSelecter(
            isMultihop,
            { isMultihop = it },
            isCollapsed,
            { isCollapsed = it },
            progress,
        )
    }
}

@OptIn(ExperimentalMotionApi::class)
@Composable
fun SimpleHopSelecter(
    isMultihop: Boolean,
    onToggleMultihop: (Boolean) -> Unit,
    isCollapsed: Boolean,
    onToggleCollapsed: (Boolean) -> Unit,
    progress: Float,
) {
    val internet = "header"
    val exit = "exit"
    val entry = "entry"
    val device = "device"

    val tSingle = "tSingle"
    val tMulti = "tMulti"
    val tCollapsed = "tCollapsed"
    val tExpanded = "tExpanded"

    val scene = MotionScene {
        val (internet, exit, entry, device) = createRefsFor(internet, exit, entry, device)

        val expanded =
            constraintSet("expanded") {
                createVerticalChain(internet, exit, device)
                constrain(entry) { centerVerticallyTo(exit) }
            }

        val collapsed =
            constraintSet("collapsed") {
                constrain(exit) { centerVerticallyTo(parent) }
                constrain(entry) { centerVerticallyTo(exit) }
                constrain(internet) { centerVerticallyTo(exit) }
                constrain(device) { centerVerticallyTo(exit) }
            }

        val expandedMulti =
            constraintSet("expandedMulti") { createVerticalChain(internet, exit, entry, device) }
        val collapsedMulti =
            constraintSet("collapsedMulti") {
                createVerticalChain(exit, entry)
                constrain(internet) { centerVerticallyTo(exit) }
                constrain(device) { centerVerticallyTo(entry) }
            }

        transition(expanded, collapsed, name = tSingle) {
            keyAttributes(internet, device) {
                frame(0) { alpha = 1f }
                frame(100) { alpha = 0f }
            }
            keyAttributes(entry) {
                frame(0) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
        }

        transition(expandedMulti, collapsedMulti, name = tMulti) {
            keyAttributes(internet, device) {
                frame(0) { alpha = 1f }
                frame(100) { alpha = 0f }
            }
        }

        transition(expanded, expandedMulti, tExpanded) {
            keyAttributes(internet, device) {
                frame(0) { alpha = 1f }
                frame(100) { alpha = 1f }
            }
            keyAttributes(entry) {
                frame(0) { alpha = 0f }
                frame(100) { alpha = 1f }
            }
        }

        transition(collapsed, collapsedMulti, tCollapsed) {
            keyAttributes(entry) {
                frame(0) { alpha = 0f }
                frame(100) { alpha = 1f }
            }
            keyAttributes(internet, device) {
                frame(0) { alpha = 0f }
                frame(100) { alpha = 0f }
            }
        }
    }

    Column {
        Text("Multihop")
        Switch(isMultihop, onToggleMultihop)
        Text("Collapsed")
        Switch(isCollapsed, onToggleCollapsed)

        val isAnimating = remember { mutableStateOf(false) }
        val target = if (isMultihop) 1f else 0f
        val animateHopChange = animateFloatAsState(target, tween(500), finishedListener = {})

        if (animateHopChange.value != target) {
            isAnimating.value = true
        } else {
            isAnimating.value = false
        }

        Text("isAnimatingHop = ${isAnimating.value}")
        Text("animateHopChange = ${animateHopChange.value}")
        Text("progress = ${progress}")

        MotionLayout(
            modifier = Modifier.fillMaxWidth().background(Color.Gray),
            motionScene = scene,
            transitionName =
                if (isAnimating.value) {
                    if (progress == 1f) tCollapsed else tExpanded
                } else {
                    if (isMultihop) tMulti else tSingle
                },
            progress = if (isAnimating.value) animateHopChange.value else progress,
        ) {
            Icon(
                modifier = Modifier.size(40.dp).layoutId(internet),
                imageVector = Icons.Default.Language,
                contentDescription = null,
            )
            Text(
                modifier = Modifier.padding(start = 8.dp).layoutId(exit),
                text = "Exit",
                style = MaterialTheme.typography.bodyMedium,
            )

            AnimatedVisibility(
                isMultihop,
                modifier = Modifier.padding(start = 8.dp).layoutId(entry),
            ) {
                Text(text = "Entry", style = MaterialTheme.typography.bodyMedium)
            }

            Icon(
                modifier = Modifier.size(40.dp).layoutId(device),
                imageVector = Icons.Default.PhoneAndroid,
                contentDescription = null,
            )
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

                val pathEffect = PathEffect.dashPathEffect(floatArrayOf(10f, 10f), 0f)
                Canvas(
                    Modifier.width(2.dp).constrainAs(dashInternetExit) {
                        height = fillToConstraints
                        centerHorizontallyTo(internetIcon)
                        top.linkTo(internetIcon.bottom)
                        bottom.linkTo(hopExit.baseline, 0.dp)
                        bottom.linkTo(hopExit.bottom, margin = (4).dp)
                    }
                ) {
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
        }
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
