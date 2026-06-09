package net.mullvad.mullvadvpn.lib.map.preview

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.AnimationEndReason
import androidx.compose.animation.core.VectorConverter
import androidx.compose.animation.core.exponentialDecay
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateCentroidSize
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateRotation
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerInputScope
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChanged
import androidx.compose.ui.input.pointer.util.VelocityTracker1D
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Velocity
import kotlin.math.PI
import kotlin.math.abs
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.map.Map
import net.mullvad.mullvadvpn.lib.map.toAnimationDurationMillis
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.model.COMPLETE_ANGLE
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

@Preview
@Composable
private fun InteractiveMapPreview() {
    // Starting position
    var currentLocation by remember {
        // Berlin
        mutableStateOf(locations[9])
    }

    // Create some markers
    val markers =
        locations.map {
            Marker(
                id = it.toString(),
                latLong = it,
                colors =
                    if (it == currentLocation) selectLocationMarkerColors
                    else unselectLocationMarkerColors,
            )
        }

    // Range of zoom levels we can zoom to, so users don't get lost in space
    val zoomRange = 1.2f..2f


    val zoomAnimatable = remember {
        Animatable(zoomRange.start).also {
            it.updateBounds(zoomRange.start, zoomRange.endInclusive)
        }
    }
    val latLngAnimatable = remember {
        Animatable(currentLocation.toOffset(), Offset.VectorConverter).also {
            // Limit Latitude to -40 to 60 degrees. There are no servers on the north or south pole, yet.
            it.updateBounds(
                lowerBound = Offset(x = Float.NEGATIVE_INFINITY, y = -40f),
                upperBound = Offset(x = Float.POSITIVE_INFINITY, y = 60f),
            )
        }
    }

    // New location selected, move camera to it
    LaunchedEffect(currentLocation) {
        // Decide duration of animation based on distance
        val distance = currentLocation.seppDistanceTo(latLngAnimatable.value.toLatLng())
        val duration = distance.toAnimationDurationMillis()

        launch {
            latLngAnimatable.snapTo(latLngAnimatable.value.unwind())
            latLngAnimatable.animateTo(currentLocation.toOffset(), animationSpec = tween(duration))
        }
        launch { zoomAnimatable.animateTo(zoomRange.start, animationSpec = tween(duration)) }
    }

    val tracker = remember { DiffVelocityTracker() }
    val scope = rememberCoroutineScope()

    val onGesture: (Offset, Offset, Float, Float) -> Unit =
        { _: Offset, pan: Offset, zoomChange: Float, _: Float ->
            // Calculate new camera position & zoom
            val currentPosition = latLngAnimatable.value
            val zoom = zoomAnimatable.value

            val latLngOffsetDiff = calculateLatLngPan(pan, zoom)
            val newPosition = currentPosition + latLngOffsetDiff

            val newZoom = (zoom + (1 - zoomChange) * 0.5f)

            // Update to the new position
            scope.launch {
                latLngAnimatable.snapTo(newPosition)
                zoomAnimatable.snapTo(newZoom)
            }

            // Track the gesture to calculate velocity later
            val isZooming = zoomChange != 1f
            if (!isZooming) {
                tracker.addPosition(System.currentTimeMillis(), latLngOffsetDiff)
            } else {
                tracker.resetTracking()
            }
        }

    val onGestureEnd: () -> Unit = {
        scope.launch {
            // Fling the map based on velocity of the gesture
            var (longVelocity, latVelocity) = tracker.calculateVelocity()
            tracker.resetTracking()
            do {
                val result =
                    latLngAnimatable.animateDecay(
                        Offset(longVelocity, latVelocity),
                        exponentialDecay(1f),
                    )

                longVelocity = result.endState.velocityVector.v1
                latVelocity = -result.endState.velocityVector.v2
            } while (result.endReason == AnimationEndReason.BoundReached)

            // Restore camera to the selected location if user doesn't select anything
            launch {
                latLngAnimatable.animateTo(
                    calculateClosestOffset(latLngAnimatable.value, currentLocation.toOffset()),
                    tween(1000, 2000),
                )
            }
            launch { zoomAnimatable.animateTo(zoomRange.start, tween(1000, 2000)) }
        }
    }

    Map(
        modifier =
            Modifier.pointerInput(
                Unit,
                {
                    detectTransformGesturesWithEnd(
                        true,
                        onGesture = onGesture,
                        onGestureEnd = onGestureEnd,
                    )
                },
            ),
        cameraPosition = CameraPosition(latLngAnimatable.value.toLatLng(), zoomAnimatable.value),
        markers = markers,
        onMarkerClick = { currentLocation = it.latLong },
    )
}
private fun calculateLatLngPan(pan: Offset, zoom: Float): Offset =
    Offset(x = -pan.x * zoom / 50f, pan.y * zoom / 40f)

private fun LatLong.toOffset(): Offset = Offset(longitude.value, latitude.value)

private fun Offset.unwind(): Offset = Offset(Longitude.unwind(x), Latitude.unwind(y))

private fun Offset.toLatLng(): LatLong = LatLong(Latitude.fromFloat(y), Longitude.fromFloat(x))

fun Float.closestTarget(target: Float): Float {
    val deg = rem(COMPLETE_ANGLE)
    val base = this - deg

    val targetRemainder = target.rem(COMPLETE_ANGLE)
    val newTarget = base + targetRemainder

    val diff = this - newTarget
    return when {
        diff > 180f -> newTarget + COMPLETE_ANGLE
        diff < -180f -> newTarget - COMPLETE_ANGLE
        else -> newTarget
    }
}

fun calculateClosestOffset(current: Offset, target: Offset): Offset =
    Offset(current.x.closestTarget(target.x), target.y)

suspend fun PointerInputScope.detectTransformGesturesWithEnd(
    panZoomLock: Boolean = false,
    onGesture: (centroid: Offset, pan: Offset, zoom: Float, rotation: Float) -> Unit,
    onGestureEnd: () -> Unit,
) {
    awaitEachGesture {
        var rotation = 0f
        var zoom = 1f
        var pan = Offset.Zero
        var pastTouchSlop = false
        val touchSlop = viewConfiguration.touchSlop
        var lockedToPanZoom = false

        awaitFirstDown(requireUnconsumed = false)
        do {
            val event = awaitPointerEvent()
            val canceled = event.changes.any { it.isConsumed }
            if (!canceled) {
                val zoomChange = event.calculateZoom()
                val rotationChange = event.calculateRotation()
                val panChange = event.calculatePan()

                if (!pastTouchSlop) {
                    zoom *= zoomChange
                    rotation += rotationChange
                    pan += panChange

                    val centroidSize = event.calculateCentroidSize(useCurrent = false)
                    val zoomMotion = abs(1 - zoom) * centroidSize
                    val rotationMotion = abs(rotation * PI.toFloat() * centroidSize / 180f)
                    val panMotion = pan.getDistance()

                    if (
                        zoomMotion > touchSlop ||
                            rotationMotion > touchSlop ||
                            panMotion > touchSlop
                    ) {
                        pastTouchSlop = true
                        lockedToPanZoom = panZoomLock && rotationMotion < touchSlop
                    }
                }

                if (pastTouchSlop) {
                    val centroid = event.calculateCentroid(useCurrent = false)
                    val effectiveRotation = if (lockedToPanZoom) 0f else rotationChange
                    if (effectiveRotation != 0f || zoomChange != 1f || panChange != Offset.Zero) {
                        onGesture(centroid, panChange, zoomChange, effectiveRotation)
                    }
                    event.changes.forEach {
                        if (it.positionChanged()) {
                            it.consume()
                        }
                    }
                }
            }
        } while (!canceled && event.changes.any { it.pressed })

        onGestureEnd()
    }
}

private val selectLocationMarkerColors =
    LocationMarkerColors(centerColor = Color(0xFF44AD4D.toInt()))

private val unselectLocationMarkerColors =
    LocationMarkerColors(
        perimeterColors = null,
        centerColor = Color(0xFF192E45.toInt()),
        ringBorderColor = Color(0xFFFFFFFF.toInt()),
    )

class DiffVelocityTracker {
    private val xVelocityTracker = VelocityTracker1D(true)
    private val yVelocityTracker = VelocityTracker1D(true)

    internal var lastMoveEventTimeStamp = 0L

    fun addPosition(timeMillis: Long, delta: Offset) {
        xVelocityTracker.addDataPoint(timeMillis, delta.x)
        yVelocityTracker.addDataPoint(timeMillis, delta.y)
    }

    fun calculateVelocity(): Velocity =
        calculateVelocity(Velocity(Float.MAX_VALUE, Float.MAX_VALUE))

    fun calculateVelocity(maximumVelocity: Velocity): Velocity {
        val velocityX = xVelocityTracker.calculateVelocity(maximumVelocity.x)
        val velocityY = yVelocityTracker.calculateVelocity(maximumVelocity.y)
        return Velocity(velocityX, velocityY)
    }

    /** Clears the tracked positions added by [addPosition]. */
    fun resetTracking() {
        xVelocityTracker.resetTracking()
        yVelocityTracker.resetTracking()
        lastMoveEventTimeStamp = 0L
    }
}
