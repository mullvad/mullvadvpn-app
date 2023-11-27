package net.mullvad.mullvadvpn.util

import kotlin.math.pow

fun Long.pow(exponent: Int): Long = toDouble().pow(exponent).toLong()
