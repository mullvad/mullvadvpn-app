package net.mullvad.mullvadvpn.lib.map.internal

internal const val VERTEX_COMPONENT_SIZE = 3
internal const val COLOR_COMPONENT_SIZE = 4
internal const val MATRIX_SIZE = 16

// Constant what will talk the distance in LatLng multiply it to determine the animation duration,
// the result is then confined to the MIN_ANIMATION_MILLIS and MAX_ANIMATION_MILLIS
const val DISTANCE_DURATION_SCALE_FACTOR = 20
const val MIN_ANIMATION_MILLIS = 1300
const val MAX_ANIMATION_MILLIS = 2500
// The cut off where we go from a short animation (camera pans) to a far animation (camera pans +
// zoom out)
const val SHORT_ANIMATION_CUTOFF_MILLIS = 1700

// Multiplier for the zoom out animation
const val FAR_ANIMATION_MAX_ZOOM_MULTIPLIER = 1.30f
// When in the far animation we reach the MAX_ZOOM_MULTIPLIER, value is between 0 and 1
const val MAX_MULTIPLIER_PEAK_TIMING = .35f
