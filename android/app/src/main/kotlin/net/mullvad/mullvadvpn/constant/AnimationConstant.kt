package net.mullvad.mullvadvpn.constant

import androidx.compose.animation.core.Spring
import net.mullvad.mullvadvpn.model.LatLong
import net.mullvad.mullvadvpn.model.Latitude
import net.mullvad.mullvadvpn.model.Longitude

const val MINIMUM_LOADING_TIME_MILLIS = 500L

const val SCREEN_ANIMATION_TIME_MILLIS = Spring.StiffnessMediumLow.toInt()

const val HORIZONTAL_SLIDE_FACTOR = 1 / 3f

fun Int.withHorizontalScalingFactor(): Int = (this * HORIZONTAL_SLIDE_FACTOR).toInt()

const val SECURE_ZOOM = 1.15f
const val UNSECURE_ZOOM = 1.20f
const val SECURE_ZOOM_ANIMATION_MILLIS = 2000

// Location of Gothenburg, Sweden
val fallbackLatLong = LatLong(Latitude(57.7065f), Longitude(11.967f))
