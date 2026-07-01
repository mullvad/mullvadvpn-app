package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlin.math.cos
import kotlin.math.pow
import kotlin.math.sqrt
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.Latitude.Companion.mean

@Parcelize
data class LatLong(val latitude: Latitude, val longitude: Longitude) : Parcelable {

    fun degreeDistanceTo(other: LatLong): Float =
        sqrt(
            latitude.distanceTo(other.latitude).pow(2f) +
                (longitude.distanceTo(other.longitude).pow(2f))
        )

    operator fun plus(other: LatLong) =
        LatLong(latitude + other.latitude, longitude + other.longitude)

    operator fun minus(other: LatLong) =
        LatLong(latitude - other.latitude, longitude - other.longitude)

    /**
     * Calculate the distance between two points on the earth's surface using the spherical earth
     * projected to a plane. ( This method has some drawbacks and shortcomings for extreme values
     * closer to the Poles but should be good enough for our use case. ) Reference:
     * https://en.wikipedia.org/wiki/Geographical_distance#Spherical_Earth_projected_to_a_plane
     *
     * @param other the other point to calculate the distance to.
     * @return the estimated distance in kilometers.
     */
    fun seppDistanceTo(other: LatLong): Float =
        EARTH_RADIUS *
            sqrt(
                latitude.distanceTo(other.latitude).toRadians().pow(2) +
                    (cos(mean(latitude, other.latitude).value.toRadians()) *
                            longitude.distanceTo(other.longitude).toRadians())
                        .pow(2)
            )

    companion object {
        // Average radius of the earth in kilometers
        const val EARTH_RADIUS = 6371.009f
    }
}

fun LatLong(latitude: Float, longitude: Float) =
    LatLong(Latitude.fromFloat(latitude), Longitude.fromFloat(longitude))

const val COMPLETE_ANGLE = 360f
const val STRAIGHT_ANGLE = 180f
const val RIGHT_ANGLE = 90f

fun Float.toRadians() = this * Math.PI.toFloat() / STRAIGHT_ANGLE

fun Float.toDegrees() = this * (STRAIGHT_ANGLE / Math.PI.toFloat())
