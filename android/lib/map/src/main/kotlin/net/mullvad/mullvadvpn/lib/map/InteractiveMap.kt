package net.mullvad.mullvadvpn.lib.map

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
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
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
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import co.touchlab.kermit.Logger
import kotlin.math.abs
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.GlobeViewState
import net.mullvad.mullvadvpn.lib.map.data.Hop
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapSurfaceView
import net.mullvad.mullvadvpn.lib.model.COMPLETE_ANGLE
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

val LAT_LOWER_BOUND = -40f
val LAT_UPPER_BOUND = 65f

@Composable
fun InteractiveMap(
    currentLocation: LatLong,
    verticalBias: Float = .5f,
    markers: List<Marker>,
    locations: List<LatLong>,
    hops: List<Hop>,
    modifier: Modifier = Modifier,
    onMarkerClick: ((Marker) -> Unit)? = null,
    globeColors: GlobeColors = GlobeColors.default(),
) {

    // Range of zoom levels we can zoom to, so users don't get lost in space
    val zoomRange = 1.2f..2.5f

    val alphaAnimation = remember {
        Animatable(0f)
    }

    val locationMarkers = locations.map {
        Marker(it, colors = LocationMarkerColors.default(alphaAnimation.value))
    }

    val zoomAnimatable = remember {
        Animatable(zoomRange.start).also {
            it.updateBounds(zoomRange.start, zoomRange.endInclusive)
        }
    }
    val latLngAnimatable = remember {
        Animatable(currentLocation.toOffset(), Offset.VectorConverter).also {
            // Limit Latitude to -40 to 60 degrees. There are no servers on the north or south pole,
            // yet.
            it.updateBounds(
                lowerBound = Offset(x = Float.NEGATIVE_INFINITY, y = LAT_LOWER_BOUND),
                upperBound = Offset(x = Float.POSITIVE_INFINITY, y = LAT_UPPER_BOUND),
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
        launch { alphaAnimation.animateTo(0f, animationSpec = tween(300)) }
    }

    val tracker = remember { DiffVelocityTracker() }
    val scope = rememberCoroutineScope()

    var view: MapSurfaceView? = remember { null }

    val onGestureStart: () -> Unit = {
        scope.launch {
            alphaAnimation.animateTo(1f, tween(500))
        }
    }

    val onGesture: (Offset, Offset, Float) -> Unit =
        onGesture@{ centroid: Offset, pan: Offset, zoomChange: Float ->
            // Calculate new camera position & zoom
            val currentPosition = latLngAnimatable.value
            val zoom = zoomAnimatable.value

            val org = view?.getPosition(centroid) ?: return@onGesture
            val new = view?.getPosition(centroid + pan) ?: return@onGesture

            val latDiff = org - new

            val newPosition =
                (currentPosition + latDiff.toOffset()).coerceIn(
                    yMin = LAT_LOWER_BOUND,
                    yMax = LAT_UPPER_BOUND,
                )
            val realDiff = newPosition - currentPosition

            Logger.d { "NewPosition: $newPosition" }
            Logger.d { "RealDiff: $realDiff" }

            val newZoom = (zoom + (1 - zoomChange) * 0.5f)

            // Update to the new position
            scope.launch {
                latLngAnimatable.snapTo(newPosition)
                zoomAnimatable.snapTo(newZoom)
            }

            // Track the gesture to calculate velocity later
            val isZooming = zoomChange != 1f
            if (!isZooming) {
                tracker.addPosition(System.currentTimeMillis(), realDiff)
            } else {
                tracker.resetTracking()
            }
        }

    val onGestureEnd: () -> Unit = {
        scope.launch {
            // Fling the map based on velocity of the gesture
            var (longVelocity, latVelocity) = tracker.calculateVelocity()
            Logger.d { "LongVelocity: $longVelocity, LatVelocity: $latVelocity" }
            Logger.d { "OnGestureEnd" }
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
            launch {
                // Ensure we animate to full alpha before starting next as animateTo will abort any
                // existing animation.
                alphaAnimation.animateTo(1f, tween(100))
                alphaAnimation.animateTo(0f, tween(400, 1900))
            }
        }
    }

    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    val cameraPosition =
        CameraPosition(
            latLngAnimatable.value.toLatLng(),
            zoomAnimatable.value,
            verticalBias = verticalBias,
        )
    val globeViewState =
        GlobeViewState(
            cameraPosition,
            markers + locationMarkers,
            hops.map { it.copy(color = Color.White.copy(alpha = alphaAnimation.value)) },
            globeColors,
        )
    AndroidView(
        modifier =
            Modifier.pointerInput(lifeCycleState) {
                    detectTapGestures(
                        onTap = {
                            val result = view?.closestMarker(it) ?: return@detectTapGestures
                            onMarkerClick?.invoke(result.first)
                        }
                    )
                }
                .pointerInput(lifeCycleState) {
                    detectTransformGesturesWithEnd(
                        onGestureStart = onGestureStart,
                        onGesture = onGesture,
                        onGestureEnd = onGestureEnd,
                    )
                },
        factory = { MapSurfaceView(it) },
        update = { glSurfaceView ->
            glSurfaceView.lifecycle = lifeCycleState
            view = glSurfaceView
            glSurfaceView.setData(globeViewState)
        },
        onRelease = { it.lifecycle = null },
    )
}

private fun Offset.coerceIn(
    xMin: Float = Float.NEGATIVE_INFINITY,
    xMax: Float = Float.POSITIVE_INFINITY,
    yMin: Float = Float.NEGATIVE_INFINITY,
    yMax: Float = Float.POSITIVE_INFINITY,
): Offset =
    Offset(
        x.coerceIn(xMin, xMax),
        y.coerceIn(yMin, yMax),
    )

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
    onGestureStart: () -> Unit,
    onGesture: (centroid: Offset, pan: Offset, zoom: Float) -> Unit,
    onGestureEnd: () -> Unit,
) {
    awaitEachGesture {
        var zoom = 1f
        var pan = Offset.Zero
        var pastTouchSlop = false
        val touchSlop = viewConfiguration.touchSlop

        awaitFirstDown(requireUnconsumed = false)
        onGestureStart()
        do {
            val event = awaitPointerEvent()
            val canceled = event.changes.any { it.isConsumed }
            if (!canceled) {
                val zoomChange = event.calculateZoom()
                val panChange = event.calculatePan()

                if (!pastTouchSlop) {
                    zoom *= zoomChange
                    pan += panChange

                    val centroidSize = event.calculateCentroidSize(useCurrent = false)
                    val zoomMotion = abs(1 - zoom) * centroidSize
                    val panMotion = pan.getDistance()

                    if (zoomMotion > touchSlop || panMotion > touchSlop) {
                        pastTouchSlop = true
                    }
                }

                if (pastTouchSlop) {
                    val centroid = event.calculateCentroid(useCurrent = false)
                    if (zoomChange != 1f || panChange != Offset.Zero) {
                        onGesture(centroid, panChange, zoomChange)
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
