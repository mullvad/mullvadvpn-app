package net.mullvad.mullvadvpn.lib.common.compose

import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

const val MINIMUM_LOADING_TIME_MILLIS = 500L

const val CONNECTED_ZOOM = 0f
const val DEFAULT_ZOOM = .05f
const val SECURE_ZOOM_ANIMATION_MILLIS = 2000

const val SETTINGS_HIGHLIGHT_REPEAT_COUNT = 3

// Location of Gothenburg, Sweden
val fallbackLatLong = LatLong(Latitude(57.7065f), Longitude(11.967f))
