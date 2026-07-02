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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.PointerInputScope
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChanged
import androidx.compose.ui.input.pointer.util.VelocityTracker1D
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import kotlin.math.abs
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.map.MapCameraController.Companion.ZOOM_RANGE
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
import net.mullvad.mullvadvpn.lib.model.STRAIGHT_ANGLE

internal class MapCameraController(
    private val scope: CoroutineScope,
    private val zoomRange: ClosedFloatingPointRange<Float> = ZOOM_RANGE,
    initialLocation: LatLong,
) {
    private var isPerformingGesture = false

    var currentLocation: LatLong by mutableStateOf(initialLocation)
        private set

    val latLngAnimatable = Animatable(initialLocation.toOffset(), Offset.VectorConverter)
    val zoomAnimatable = Animatable(zoomRange.start)
    val alphaAnimation = Animatable(0f)

    private val tracker = DiffVelocityTracker()
    private var returnToIdleJob: Job? = null

    init {
        latLngAnimatable.updateBounds(
            lowerBound = Offset(x = Float.NEGATIVE_INFINITY, y = LAT_LOWER_BOUND),
            upperBound = Offset(x = Float.POSITIVE_INFINITY, y = LAT_UPPER_BOUND),
        )
        zoomAnimatable.updateBounds(zoomRange.start, zoomRange.endInclusive)
    }

    fun onCurrentLocationChanged(newLocation: LatLong) {
        currentLocation = newLocation
        if (!isPerformingGesture) {
            cancelReturnJob()
            animateToLocation(newLocation)
        }
    }

    private fun animateToLocation(target: LatLong) {
        scope.launch {
            val distance = target.seppDistanceTo(latLngAnimatable.value.toLatLng())
            val duration = distance.toAnimationDurationMillis()

            launch {
                latLngAnimatable.snapTo(latLngAnimatable.value.unwind())
                latLngAnimatable.animateTo(target.toOffset(), animationSpec = tween(duration))
            }
            launch { zoomAnimatable.animateTo(zoomRange.start, animationSpec = tween(duration)) }
            launch {
                alphaAnimation.animateTo(0f, animationSpec = tween(INTERACTIVE_FADE_OUT_DURATION))
            }
        }
    }

    fun onGestureStart() {
        cancelReturnJob()
        isPerformingGesture = true
        scope.launch {
            zoomAnimatable.stop()
            latLngAnimatable.stop()
            alphaAnimation.animateTo(1f, tween(INTERACTIVE_FADE_IN_DURATION))
        }
    }

    fun onGesture(
        centroid: Offset,
        pan: Offset,
        zoomChange: Float,
        offsetToLatLng: (Offset) -> LatLong?,
    ) {
        val currentPosition = latLngAnimatable.value
        val zoom = zoomAnimatable.value

        val org = offsetToLatLng(centroid) ?: return
        val new = offsetToLatLng(centroid + pan) ?: return

        val latDiff = org - new

        val newPosition =
            (currentPosition + latDiff.toOffset()).coerceIn(
                yMin = LAT_LOWER_BOUND,
                yMax = LAT_UPPER_BOUND,
            )
        val realDiff = newPosition - currentPosition
        val newZoom = (zoom + (1 - zoomChange) * 0.5f)

        scope.launch {
            latLngAnimatable.snapTo(newPosition)
            zoomAnimatable.snapTo(newZoom)
        }

        val isZooming = zoomChange != 1f
        if (!isZooming) {
            tracker.addPosition(System.currentTimeMillis(), realDiff)
        } else {
            tracker.resetTracking()
        }
    }

    fun onGestureEnd() {
        isPerformingGesture = false
        returnToIdleJob = scope.launch {
            var (longVelocity, latVelocity) =
                tracker.calculateVelocity(
                    maximumVelocity = Velocity(MAX_FLING_VELOCITY, MAX_FLING_VELOCITY)
                )
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

            delay(RESTORE_CAMERA_TIMEOUT)

            launch {
                latLngAnimatable.animateTo(
                    calculateClosestOffset(latLngAnimatable.value, currentLocation.toOffset()),
                    tween(RESTORE_CAMERA_POSITION_DURATION),
                )
            }
            launch {
                zoomAnimatable.animateTo(zoomRange.start, tween(RESTORE_CAMERA_POSITION_DURATION))
            }
            launch { alphaAnimation.animateTo(0f, tween(INTERACTIVE_FADE_OUT_DURATION)) }
        }
    }

    private fun cancelReturnJob() {
        returnToIdleJob?.cancel()
        returnToIdleJob = null
    }

    companion object {

        // How far up the globe the camera can view
        private const val LAT_LOWER_BOUND = -40f
        private const val LAT_UPPER_BOUND = 65f

        // 1f is the surface of the globe
        internal val ZOOM_RANGE: ClosedFloatingPointRange<Float> = 1.2f..2.5f

        private const val MAX_FLING_VELOCITY = 1000f

        private val RESTORE_CAMERA_TIMEOUT = 3.seconds
        private const val RESTORE_CAMERA_POSITION_DURATION = 1000

        private const val INTERACTIVE_FADE_IN_DURATION = 400
        private const val INTERACTIVE_FADE_OUT_DURATION = 400
    }
}

@Composable
internal fun rememberMapCameraController(
    currentLocation: LatLong,
    zoomRange: ClosedFloatingPointRange<Float> = ZOOM_RANGE,
    scope: CoroutineScope = rememberCoroutineScope(),
): MapCameraController {
    val controller = remember {
        MapCameraController(scope, zoomRange, currentLocation)
    }
    LaunchedEffect(currentLocation) {
        controller.onCurrentLocationChanged(currentLocation)
    }
    return controller
}

@Composable
fun InteractiveMap(
    currentLocation: LatLong,
    secureZoom: Float = 0f,
    verticalBias: Float = .5f,
    markers: List<Marker>,
    locations: List<LatLong>,
    hops: List<Hop>,
    onMarkerClick: ((Marker) -> Unit)? = null,
    globeColors: GlobeColors = GlobeColors.default(),
) {
    val controller = rememberMapCameraController(currentLocation)

    val hopMarkers = hops.map {
        Marker(
            it.from,
            colors = LocationMarkerColors.hop(alpha = controller.alphaAnimation.value),
        )
    }
    val locationMarkers = locations.map {
        Marker(it, colors = LocationMarkerColors.default(controller.alphaAnimation.value))
    }
    var view: MapSurfaceView? = remember { null }

    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    val cameraPosition =
        CameraPosition(
            controller.latLngAnimatable.value.toLatLng(),
            controller.zoomAnimatable.value + secureZoom,
            verticalBias = verticalBias,
        )
    val globeViewState =
        GlobeViewState(
            cameraPosition,
            markers + hopMarkers + locationMarkers,
            hops.map {
                it.copy(
                    color = it.color.copy(alpha = it.color.alpha * controller.alphaAnimation.value)
                )
            },
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
                        onGestureStart = { controller.onGestureStart() },
                        onGesture = { centroid, pan, zoom ->
                            controller.onGesture(centroid, pan, zoom, { view!!.getPosition(it) })
                        },
                        onGestureEnd = { controller.onGestureEnd() },
                    )
                },
        factory = { MapSurfaceView(it) },
        update = { glSurfaceView ->
            glSurfaceView.lifecycle = lifeCycleState
            view = glSurfaceView
            glSurfaceView.setData(globeViewState)
        },
        onRelease = {
            it.lifecycle = null
        },
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
        diff > STRAIGHT_ANGLE -> newTarget + COMPLETE_ANGLE
        diff < -STRAIGHT_ANGLE -> newTarget - COMPLETE_ANGLE
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
