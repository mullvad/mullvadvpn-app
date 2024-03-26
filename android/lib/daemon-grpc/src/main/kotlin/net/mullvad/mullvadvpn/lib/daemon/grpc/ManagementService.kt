package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.LocalSocketAddress
import android.net.Uri
import android.util.Log
import com.google.protobuf.Empty
import com.google.protobuf.StringValue
import io.grpc.Status
import io.grpc.StatusException
import io.grpc.android.UdsChannelBuilder
import java.net.InetAddress
import java.net.InetSocketAddress
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import mullvad_daemon.management_interface.ManagementInterface
import mullvad_daemon.management_interface.ManagementInterface.*
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory as ModelAccountHistory
import net.mullvad.mullvadvpn.model.AccountState
import net.mullvad.mullvadvpn.model.Device as ModelDevice
import net.mullvad.mullvadvpn.model.GeoIpLocation as ModelGeoIpLocation
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.model.TunnelState as ModelTunnelState
import net.mullvad.talpid.net.Endpoint as ModelEndpoint
import net.mullvad.talpid.net.ObfuscationEndpoint as ModelObfuscationEndpoint
import net.mullvad.talpid.net.ObfuscationType as ModelObfuscationType
import net.mullvad.talpid.net.TransportProtocol as ModelTransportProtocol
import net.mullvad.talpid.net.TunnelEndpoint as ModelTunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState as ModelErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause as ModelErrorStateCause
import net.mullvad.talpid.tunnel.FirewallPolicyError as ModelFirewallPolicyError
import net.mullvad.talpid.tunnel.ParameterGenerationError as ModelParameterGenerationError
import net.mullvad.mullvadvpn.model.Settings as ModelSettings
import org.joda.time.Instant

class ManagementService(
    rpcSocketPath: String,
    private val scope: CoroutineScope,
) {

    data class ManagementServiceState(
        val tunnelState: TunnelState? = null,
        val settings: Settings? = null,
        val relayList: RelayList? = null,
        val versionInfo: AppVersionInfo? = null,
        val device: DeviceState? = null,
        val deviceEvent: DeviceEvent? = null,
    )

    private val channel =
        UdsChannelBuilder.forPath(rpcSocketPath, LocalSocketAddress.Namespace.FILESYSTEM).build()
    private val managementService =
        ManagementServiceGrpcKt.ManagementServiceCoroutineStub(channel).withWaitForReady()

    private val _mutableStateFlow: MutableStateFlow<ManagementServiceState> =
        MutableStateFlow(ManagementServiceState())
    val state: StateFlow<ManagementServiceState> = _mutableStateFlow

    val deviceState: Flow<AccountState> =
        _mutableStateFlow
            .mapNotNull { it.device }
            .map {
                when (it.state) {
                    DeviceState.State.LOGGED_IN ->
                        AccountState.LoggedIn(
                            device =
                                ModelDevice(
                                    it.device.device.id,
                                    it.device.device.name,
                                    it.device.device.pubkey.toByteArray(),
                                    it.device.device.created.toString(),
                                ),
                            accountToken = it.device.accountToken
                        )
                    DeviceState.State.LOGGED_OUT -> AccountState.LoggedOut
                    DeviceState.State.REVOKED -> AccountState.Revoked
                    DeviceState.State.UNRECOGNIZED -> AccountState.Unrecognized
                }
            }

    val tunnelState: Flow<ModelTunnelState> =
        _mutableStateFlow.mapNotNull { it.tunnelState }.map { it.toTunnelState() }

    val settings: Flow<ModelSettings> =
        _mutableStateFlow.mapNotNull { it.settings }.map { it.toSettings() }

    suspend fun start() {
        scope.launch { _mutableStateFlow.update { getInitialServiceState() } }
        scope.launch {
            managementService.eventsListen(Empty.getDefaultInstance()).collect { event ->
                @Suppress("WHEN_ENUM_CAN_BE_NULL_IN_JAVA")
                when (event.eventCase) {
                    DaemonEvent.EventCase.TUNNEL_STATE ->
                        _mutableStateFlow.update { it.copy(tunnelState = event.tunnelState) }
                    DaemonEvent.EventCase.SETTINGS ->
                        _mutableStateFlow.update { it.copy(settings = event.settings) }
                    DaemonEvent.EventCase.RELAY_LIST ->
                        _mutableStateFlow.update { it.copy(relayList = event.relayList) }
                    DaemonEvent.EventCase.VERSION_INFO ->
                        _mutableStateFlow.update { it.copy(versionInfo = event.versionInfo) }
                    DaemonEvent.EventCase.DEVICE ->
                        _mutableStateFlow.update { it.copy(device = event.device.newState) }
                    DaemonEvent.EventCase.REMOVE_DEVICE -> {}
                    DaemonEvent.EventCase.EVENT_NOT_SET -> {}
                    DaemonEvent.EventCase.NEW_ACCESS_METHOD -> {}
                }
            }
        }
    }

    suspend fun getDevice(): DeviceState = managementService.getDevice(Empty.getDefaultInstance())

    suspend fun getTunnelState(): TunnelState =
        managementService.getTunnelState(Empty.getDefaultInstance())

    suspend fun connect(): Boolean =
        managementService.connectTunnel(Empty.getDefaultInstance()).value

    suspend fun disconnect(): Boolean =
        managementService.disconnectTunnel(Empty.getDefaultInstance()).value

    suspend fun reconnect(): Boolean =
        managementService.reconnectTunnel(Empty.getDefaultInstance()).value

    suspend fun getSettings(): Settings = managementService.getSettings(Empty.getDefaultInstance())

    suspend fun getRelayList(): RelayList =
        managementService.getRelayLocations(Empty.getDefaultInstance())

    suspend fun getVersionInfo(): AppVersionInfo =
        managementService.getVersionInfo(Empty.getDefaultInstance())

    suspend fun logoutAccount(): Unit {
        managementService.logoutAccount(Empty.getDefaultInstance())
    }

    suspend fun loginAccount(accountToken: String): LoginResult {
        return try {
            managementService.loginAccount(StringValue.of(accountToken))
            LoginResult.Ok
        } catch (e: StatusException) {
            when (e.status.code) {
                Status.Code.OK -> TODO()
                Status.Code.RESOURCE_EXHAUSTED -> LoginResult.MaxDevicesReached
                Status.Code.UNAVAILABLE -> LoginResult.RpcError
                Status.Code.UNAUTHENTICATED -> LoginResult.InvalidAccount
                Status.Code.CANCELLED -> TODO()
                Status.Code.UNKNOWN -> TODO()
                Status.Code.INVALID_ARGUMENT -> TODO()
                Status.Code.DEADLINE_EXCEEDED -> TODO()
                Status.Code.NOT_FOUND -> TODO()
                Status.Code.ALREADY_EXISTS -> TODO()
                Status.Code.PERMISSION_DENIED -> TODO()
                Status.Code.FAILED_PRECONDITION -> TODO()
                Status.Code.ABORTED -> TODO()
                Status.Code.OUT_OF_RANGE -> TODO()
                Status.Code.UNIMPLEMENTED -> TODO()
                Status.Code.INTERNAL -> TODO()
                Status.Code.DATA_LOSS -> TODO()
            }
        }
    }

    suspend fun clearAccountHistory(): Unit {
        managementService.clearAccountHistory(Empty.getDefaultInstance())
    }

    suspend fun getAccountHistory() =
        try {
            val history = managementService.getAccountHistory(Empty.getDefaultInstance())
            if (history.hasToken()) {
                ModelAccountHistory.Available(history.token.value)
            } else {
                ModelAccountHistory.Missing
            }
        } catch (e: StatusException) {
            throw e
        }

    private suspend fun getInitialServiceState() =
        ManagementServiceState(
            getTunnelState(),
            getSettings(),
            getRelayList(),
            getVersionInfo(),
            getDevice(),
        )

    suspend fun getAccountExpiry(accountToken: String): AccountExpiry =
        try {
            val expiry = managementService.getAccountData(StringValue.of(accountToken))
            if (expiry.hasExpiry()) {
                AccountExpiry.Available(Instant.ofEpochSecond(expiry.expiry.seconds).toDateTime())
            } else {
                AccountExpiry.Missing
            }
        } catch (e: StatusException) {
            throw e
        }

    suspend fun createAccount(): AccountCreationResult =
        try {
            val accountTokenStringValue =
                managementService.createNewAccount(Empty.getDefaultInstance())
            AccountCreationResult.Success(accountTokenStringValue.value)
        } catch (e: StatusException) {
            Log.e("ManagementService", "createAccount error: ${e.message}")
            AccountCreationResult.Failure
        }
}

fun TunnelState.toTunnelState(): ModelTunnelState =
    when (stateCase!!) {
        TunnelState.StateCase.DISCONNECTED ->
            ModelTunnelState.Disconnected(
                location = disconnected.disconnectedLocation.toLocation(),
            )
        TunnelState.StateCase.CONNECTING ->
            ModelTunnelState.Connecting(
                endpoint = connecting.relayInfo.tunnelEndpoint.toTunnelEndpoint(),
                location = connecting.relayInfo.location.toLocation(),
            )
        TunnelState.StateCase.CONNECTED ->
            ModelTunnelState.Connected(
                endpoint = connected.relayInfo.tunnelEndpoint.toTunnelEndpoint(),
                location = connected.relayInfo.location.toLocation(),
            )
        TunnelState.StateCase.DISCONNECTING ->
            ModelTunnelState.Disconnecting(
                actionAfterDisconnect = disconnecting.afterDisconnect.toActionAfterDisconnect(),
            )
        TunnelState.StateCase.ERROR ->
            ModelTunnelState.Error(errorState = error.errorState.toErrorState())
        TunnelState.StateCase.STATE_NOT_SET ->
            ModelTunnelState.Disconnected(
                location = disconnected.disconnectedLocation.toLocation(),
            )
    }

fun GeoIpLocation.toLocation(): ModelGeoIpLocation =
    ModelGeoIpLocation(
        ipv4 = InetAddress.getByName(this.ipv4),
        ipv6 = InetAddress.getByName(this.ipv6),
        country = this.country,
        city = this.city,
        latitude = this.latitude,
        longitude = this.longitude,
        hostname = this.hostname
    )

fun TunnelEndpoint.toTunnelEndpoint(): ModelTunnelEndpoint =
    ModelTunnelEndpoint(
        endpoint =
            with(address.split(":")) {
                ModelEndpoint(
                    address = InetSocketAddress(this[0], this[1].toInt()),
                    protocol = this@toTunnelEndpoint.protocol.toTransportProtocol()
                )
            },
        quantumResistant = this.quantumResistant,
        obfuscation = this.obfuscation.toObfuscationEndpoint()
    )

fun ObfuscationEndpoint.toObfuscationEndpoint(): ModelObfuscationEndpoint =
    ModelObfuscationEndpoint(
        endpoint =
            ModelEndpoint(
                address = InetSocketAddress(address, port),
                protocol = this.protocol.toTransportProtocol()
            ),
        obfuscationType = this.obfuscationType.toObfusationType()
    )

fun ObfuscationType.toObfusationType(): ModelObfuscationType =
    when (this) {
        ObfuscationType.UDP2TCP -> ModelObfuscationType.Udp2Tcp
        ObfuscationType.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized obfuscation type")
    }

fun Endpoint.toEndpoint(): ModelEndpoint =
    ModelEndpoint(
        address = with(Uri.parse(this.address)) { InetSocketAddress(host, port) },
        protocol = this.protocol.toTransportProtocol()
    )

fun TransportProtocol.toTransportProtocol(): ModelTransportProtocol =
    when (this) {
        TransportProtocol.TCP -> ModelTransportProtocol.Tcp
        TransportProtocol.UDP -> ModelTransportProtocol.Udp
        TransportProtocol.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized transport protocol")
    }

fun AfterDisconnect.toActionAfterDisconnect(): ActionAfterDisconnect =
    when (this) {
        AfterDisconnect.NOTHING -> ActionAfterDisconnect.Nothing
        AfterDisconnect.RECONNECT -> ActionAfterDisconnect.Reconnect
        AfterDisconnect.BLOCK -> ActionAfterDisconnect.Block
        AfterDisconnect.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized action after disconnect")
    }

fun ErrorState.toErrorState(): ModelErrorState =
    ModelErrorState(cause = this.toErrorStateCause(), isBlocking = this.hasBlockingError())

fun ManagementInterface.ErrorState.toErrorStateCause(): ModelErrorStateCause =
    when (this.cause!!) {
        ErrorState.Cause.AUTH_FAILED -> ModelErrorStateCause.AuthFailed(this.authFailedError.name)
        ErrorState.Cause.IPV6_UNAVAILABLE -> ModelErrorStateCause.Ipv6Unavailable
        ErrorState.Cause.SET_FIREWALL_POLICY_ERROR ->
            ModelErrorStateCause.SetFirewallPolicyError(this.policyError.toFirewallPolicyError())
        ErrorState.Cause.SET_DNS_ERROR -> ModelErrorStateCause.SetDnsError
        ErrorState.Cause.START_TUNNEL_ERROR -> ModelErrorStateCause.StartTunnelError
        ErrorState.Cause.TUNNEL_PARAMETER_ERROR ->
            ModelErrorStateCause.TunnelParameterError(
                this.parameterError.toParameterGenerationError()
            )
        ErrorState.Cause.IS_OFFLINE -> ModelErrorStateCause.IsOffline
        ErrorState.Cause.VPN_PERMISSION_DENIED -> ModelErrorStateCause.VpnPermissionDenied
        ErrorState.Cause.SPLIT_TUNNEL_ERROR -> ModelErrorStateCause.StartTunnelError
        ErrorState.Cause.UNRECOGNIZED,
        ErrorState.Cause.CREATE_TUNNEL_DEVICE ->
            throw IllegalArgumentException("Unrecognized error state cause")
    }

fun ErrorState.FirewallPolicyError.toFirewallPolicyError(): ModelFirewallPolicyError =
    when (this.type!!) {
        ErrorState.FirewallPolicyError.ErrorType.GENERIC -> ModelFirewallPolicyError.Generic
        ErrorState.FirewallPolicyError.ErrorType.LOCKED,
        ErrorState.FirewallPolicyError.ErrorType.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized firewall policy error")
    }

fun ManagementInterface.ErrorState.GenerationError.toParameterGenerationError():
    ModelParameterGenerationError =
    when (this) {
        ErrorState.GenerationError.NO_MATCHING_RELAY ->
            ModelParameterGenerationError.NoMatchingRelay
        ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY ->
            ModelParameterGenerationError.NoMatchingBridgeRelay
        ErrorState.GenerationError.NO_WIREGUARD_KEY -> ModelParameterGenerationError.NoWireguardKey
        ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR ->
            ModelParameterGenerationError.CustomTunnelHostResultionError
        ErrorState.GenerationError.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized parameter generation error")
    }
