package net.mullvad.mullvadvpn.test.benchmark.api.connectioncheck

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ConnCheckResult(
    @SerialName("mullvad_exit_ip") val mullvadExitIp: Boolean,
    val ip: String,
    val organization: String,
    val country: String,
    val city: String,
    val longitude: Double,
    val latitude: Double,
)
