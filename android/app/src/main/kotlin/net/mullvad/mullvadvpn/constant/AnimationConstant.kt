package net.mullvad.mullvadvpn.constant

import androidx.compose.animation.core.Spring

const val MINIMUM_LOADING_TIME_MILLIS = 500L

const val SCREEN_ANIMATION_TIME_MILLIS = Spring.StiffnessMediumLow.toInt()

const val HORIZONTAL_SLIDE_FACTOR = 1 / 3f

fun Int.withHorizontalScalingFactor(): Int = (this * HORIZONTAL_SLIDE_FACTOR).toInt()
