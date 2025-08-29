package net.mullvad.mullvadvpn.constant

import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

const val MINIMUM_LOADING_TIME_MILLIS = 500L

const val ENTER_TRANSITION_SCALE_IN_FACTOR = 1.1f
const val ENTER_TRANSITION_SLIDE_FACTOR = 0.25f
const val EXIT_TRANSITION_SLIDE_FACTOR = 0.1f

const val SECURE_ZOOM = 1.15f
const val UNSECURE_ZOOM = 1.20f
const val SECURE_ZOOM_ANIMATION_MILLIS = 2000

const val SETTINGS_HIGHLIGHT_REPEAT_COUNT = 3

// Location of Gothenburg, Sweden
val fallbackLatLong = LatLong(Latitude(57.7065f), Longitude(11.967f))
