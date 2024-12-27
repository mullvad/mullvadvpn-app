package net.mullvad.mullvadvpn.lib.model.map

import kotlin.math.acos
import kotlin.math.atan
import kotlin.math.cos
import kotlin.math.sin
import kotlin.math.sqrt
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude
import net.mullvad.mullvadvpn.lib.model.toDegrees
import net.mullvad.mullvadvpn.lib.model.toRadians
import kotlin.math.atan2

data class Vector3(val x: Float, val y: Float, val z: Float) {
    fun dot(other: Vector3): Float {
        return x * other.x + y * other.y + z * other.z
    }

    operator fun minus(other: Vector3): Vector3 {
        return Vector3(x - other.x, y - other.y, z - other.z)
    }

    operator fun times(scalar: Float): Vector3 {
        return Vector3(x * scalar, y * scalar, z * scalar)
    }

    operator fun plus(other: Vector3): Vector3 {
        return Vector3(x + other.x, y + other.y, z + other.z)
    }

    fun normalize(): Vector3 {
        val length = sqrt(x * x + y * y + z * z)
        return Vector3(x / length, y / length, z / length)
    }

    fun distanceTo(other: Vector3): Float {
        val dx = x - other.x
        val dy = y - other.y
        val dz = z - other.z
        return sqrt(dx * dx + dy * dy + dz * dz)
    }
}

fun Vector3.rotateAroundX(degrees: Float): Vector3 {
    val radians = degrees.toRadians()
    val cosTheta = cos(radians)
    val sinTheta = sin(radians)

    val newY = cosTheta * y - sinTheta * z
    val newZ = sinTheta * y + cosTheta * z

    return Vector3(x, -newY, newZ)
}

fun Vector3.rotateAroundY(degrees: Float): Vector3 {
    val radians = degrees.toRadians()
    val cosTheta = cos(radians)
    val sinTheta = sin(radians)

    val newX = cosTheta * x + sinTheta * z
    val newZ = -sinTheta * x + cosTheta * z

    return Vector3(newX, y, -newZ)
}

fun Vector3.toLatLng(): LatLong {
    // phi
    val lat = acos(y / Sphere.RADIUS)

    // theta
    val lon = atan2(x, z)

    return LatLong(
        // This worked for some reason (camera starts at lat 90!)
        Latitude.fromFloat(90f - lat.toDegrees()),
        Longitude.fromFloat(-lon.toDegrees()),
    )
}

fun LatLong.toVector3(): Vector3 {
    val phi = this.latitude.value.toRadians()
    val theta = this.longitude.value.toRadians()

    val x = -Sphere.RADIUS * cos(phi) * sin(theta)
    val y = Sphere.RADIUS * sin(phi)
    val z = Sphere.RADIUS * cos(phi) * cos(theta)

    return Vector3(x, y, z)
}
