package net.mullvad.mullvadvpn.lib.model.map

data class Sphere(val center: Vector3, val radius: Float) {
    companion object {
        const val RADIUS = 1f
    }
}
