package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.LocalSocketAddress
import android.net.Uri
import android.util.Log
import com.google.protobuf.BoolValue
import com.google.protobuf.Empty
import com.google.protobuf.StringValue
import com.google.protobuf.UInt32Value
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
import mullvad_daemon.management_interface.ManagementInterface.DnsOptions.DnsState
import mullvad_daemon.management_interface.ManagementInterface.ObfuscationSettings.SelectedObfuscation
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
import mullvad_daemon.management_interface.copy
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory as ModelAccountHistory
import net.mullvad.mullvadvpn.model.AccountState
import net.mullvad.mullvadvpn.model.AppVersionInfo as ModelAppVersionInfo
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomDnsOptions as ModelCustomDnsOptions
import net.mullvad.mullvadvpn.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.model.CustomListsSettings
import net.mullvad.mullvadvpn.model.DefaultDnsOptions as ModelDefaultDnsOptions
import net.mullvad.mullvadvpn.model.Device as ModelDevice
import net.mullvad.mullvadvpn.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.model.GeoIpLocation as ModelGeoIpLocation
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint as ModelGeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint as ModelLocationConstraint
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.model.RelayConstraints as ModelRelayConstraint
import net.mullvad.mullvadvpn.model.RelaySettings as ModelRelaySettings
import net.mullvad.mullvadvpn.model.SelectedObfuscation as ModelSelectedObfuscation
import net.mullvad.mullvadvpn.model.Settings as ModelSettings
import net.mullvad.mullvadvpn.model.TunnelOptions as ModelTunnelOptions
import net.mullvad.mullvadvpn.model.TunnelState as ModelTunnelState
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings as ModelUdp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.model.WireguardConstraints as ModelWireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions as ModelWireguardTunnelOptions
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
        _mutableStateFlow.mapNotNull { it.settings }.map { it.toModelSettings() }

    val versionInfo: Flow<ModelAppVersionInfo> =
        _mutableStateFlow.mapNotNull { it.versionInfo }.map { it.toModelAppVersionInfo() }

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

    suspend fun setDnsOptions(dnsOptions: ModelDnsOptions) {
        managementService.setDnsOptions(dnsOptions.toDnsOptions())
    }

    suspend fun setDnsState(dnsState: ModelDnsState) {
        val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
        val newDnsState = dnsState.toDnsState()
        managementService.setDnsOptions(currentDnsOptions.copy { this.state = newDnsState })
    }

    suspend fun setCustomDns(index: Int, address: InetAddress) {
        val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
        managementService.setDnsOptions(
            currentDnsOptions.also { it.customOptions.addressesList[index] = address.toString() }
        )
    }

    suspend fun deleteCustomDns(address: InetAddress) {
        val currentDnsOptions = getSettings().tunnelOptions.dnsOptions
        val currentCustomDnsOptions = currentDnsOptions.customOptions
        val newCustomDnsOptions =
            CustomDnsOptions.newBuilder()
                .addAllAddresses(
                    currentCustomDnsOptions.addressesList.filter { it != address.toString() }
                )
                .build()
        managementService.setDnsOptions(
            currentDnsOptions.copy { this.customOptions = newCustomDnsOptions }
        )
    }

    suspend fun setWireguardMtu(value: Int) {
        managementService.setWireguardMtu(UInt32Value.of(value))
    }

    suspend fun setWireguardQuantumResistant(value: ModelQuantumResistantState) {
        managementService.setQuantumResistantTunnel(value.toQuantumResistantState())
    }

    suspend fun setObfuscationOptions(value: ModelObfuscationSettings) {
        managementService.setObfuscationSettings(value.toObfuscationSettings())
    }

    suspend fun setAutoConnect(isEnabled: Boolean) {
        managementService.setAutoConnect(BoolValue.of(isEnabled))
    }

    suspend fun setAllowLan(allow: Boolean) {
        managementService.setAllowLan(BoolValue.of(allow))
    }

    suspend fun getCurrentVersion(): String =
        managementService.getCurrentVersion(Empty.getDefaultInstance()).value
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

fun ManagementInterface.Settings.toModelSettings(): ModelSettings =
    ModelSettings(
        relaySettings = relaySettings.toModelRelaySettings(),
        obfuscationSettings = obfuscationSettings.toObfuscationSettings(),
        customLists = CustomListsSettings(customLists.customListsList.map { it.toCustomList() }),
        allowLan = allowLan,
        autoConnect = autoConnect,
        tunnelOptions = tunnelOptions.toTunnelOptions(),
        showBetaReleases = showBetaReleases
    )

fun ManagementInterface.RelaySettings.toModelRelaySettings(): ModelRelaySettings =
    when (endpointCase) {
        RelaySettings.EndpointCase.CUSTOM -> ModelRelaySettings.CustomTunnelEndpoint
        RelaySettings.EndpointCase.NORMAL ->
            ModelRelaySettings.Normal(this.normal.toRelayLocationConstraint())
        RelaySettings.EndpointCase.ENDPOINT_NOT_SET ->
            throw IllegalArgumentException("RelaySettings endpoint not set")
    }

fun NormalRelaySettings.toRelayLocationConstraint(): ModelRelayConstraint =
    ModelRelayConstraint(
        location = location.toLocationConstraint(),
        providers = providersList.toProviderConstraint(),
        ownership = ownership.toOwnershipConstraint(),
        wireguardConstraints = wireguardConstraints.toWireguardConstraints()
    )

fun LocationConstraint.toLocationConstraint(): Constraint<ModelLocationConstraint> =
    when (typeCase) {
        LocationConstraint.TypeCase.CUSTOM_LIST ->
            Constraint.Only(ModelLocationConstraint.CustomList(customList))
        LocationConstraint.TypeCase.LOCATION ->
            Constraint.Only(
                ModelLocationConstraint.Location(location.toGeographicLocationConstraint())
            )
        LocationConstraint.TypeCase.TYPE_NOT_SET -> Constraint.Any()
    }

fun GeographicLocationConstraint.toGeographicLocationConstraint():
    ModelGeographicLocationConstraint =
    when {
        hasHostname() && hasCity() ->
            ModelGeographicLocationConstraint.Hostname(country, city, hostname)
        hasCity() -> ModelGeographicLocationConstraint.City(country, city)
        else -> ModelGeographicLocationConstraint.Country(country)
    }

fun List<String>.toProviderConstraint(): Constraint<Providers> =
    if (isEmpty()) Constraint.Any() else Constraint.Only(Providers(HashSet(this)))

fun WireguardConstraints.toWireguardConstraints(): ModelWireguardConstraints =
    ModelWireguardConstraints(
        port =
            if (hasPort()) {
                Constraint.Any()
            } else {
                Constraint.Only(Port(port))
            },
    )

fun Ownership.toOwnershipConstraint(): Constraint<ModelOwnership> =
    when (this) {
        Ownership.ANY -> Constraint.Any()
        Ownership.MULLVAD_OWNED -> Constraint.Only(ModelOwnership.MullvadOwned)
        Ownership.RENTED -> Constraint.Only(ModelOwnership.Rented)
        Ownership.UNRECOGNIZED -> throw IllegalArgumentException("Unrecognized ownership")
    }

fun ObfuscationSettings.toObfuscationSettings(): ModelObfuscationSettings =
    ModelObfuscationSettings(
        selectedObfuscation = selectedObfuscation.toSelectedObfuscation(),
        udp2tcp = this.udp2Tcp.toUdp2TcpObfuscationSettings()
    )

fun SelectedObfuscation.toSelectedObfuscation(): ModelSelectedObfuscation =
    when (this) {
        SelectedObfuscation.AUTO -> ModelSelectedObfuscation.Auto
        SelectedObfuscation.OFF -> ModelSelectedObfuscation.Off
        SelectedObfuscation.UDP2TCP -> ModelSelectedObfuscation.Udp2Tcp
        SelectedObfuscation.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized selected obfuscation")
    }

fun Udp2TcpObfuscationSettings.toUdp2TcpObfuscationSettings(): ModelUdp2TcpObfuscationSettings =
    if (this.hasPort()) {
        ModelUdp2TcpObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        ModelUdp2TcpObfuscationSettings(Constraint.Any())
    }

fun CustomList.toCustomList(): ModelCustomList =
    ModelCustomList(
        id = id,
        name = name,
        locations = locationsList.map { it.toGeographicLocationConstraint() }
    )

fun TunnelOptions.toTunnelOptions(): ModelTunnelOptions =
    ModelTunnelOptions(
        wireguard = wireguard.toWireguardTunnelOptions(),
        dnsOptions = dnsOptions.toDnsOptions()
    )

private fun TunnelOptions.WireguardOptions.toWireguardTunnelOptions(): ModelWireguardTunnelOptions =
    ModelWireguardTunnelOptions(
        mtu = if (hasMtu()) mtu else null,
        quantumResistant = this.quantumResistant.toQuantumResistantState(),
    )

fun QuantumResistantState.toQuantumResistantState(): ModelQuantumResistantState =
    when (state) {
        QuantumResistantState.State.AUTO -> ModelQuantumResistantState.Auto
        QuantumResistantState.State.ON -> ModelQuantumResistantState.On
        QuantumResistantState.State.OFF -> ModelQuantumResistantState.Off
        QuantumResistantState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized quantum resistant state")
    }

fun DnsOptions.toDnsOptions(): ModelDnsOptions =
    ModelDnsOptions(
        state = this.state.toDnsState(),
        defaultOptions = defaultOptions.toDefaultDnsOptions(),
        customOptions = customOptions.toCustomDnsOptions()
    )

fun DnsState.toDnsState(): ModelDnsState =
    when (this) {
        DnsState.DEFAULT -> ModelDnsState.Default
        DnsState.CUSTOM -> ModelDnsState.Custom
        DnsState.UNRECOGNIZED -> throw IllegalArgumentException("Unrecognized dns state")
    }

fun DefaultDnsOptions.toDefaultDnsOptions() =
    net.mullvad.mullvadvpn.model.DefaultDnsOptions(
        blockAds = blockAds,
        blockMalware = blockMalware,
        blockAdultContent = blockAdultContent,
        blockGambling = blockGambling,
        blockSocialMedia = blockSocialMedia,
        blockTrackers = blockTrackers
    )

fun CustomDnsOptions.toCustomDnsOptions() =
    ModelCustomDnsOptions(this.addressesList.map { InetAddress.getByName(it) })

fun ModelDnsOptions.toDnsOptions(): DnsOptions =
    DnsOptions.newBuilder()
        .setState(this.state.toDnsState())
        .setCustomOptions(this.customOptions.toCustomOptions())
        .setDefaultOptions(this.defaultOptions.toDefaultOptions())
        .build()

fun ModelDnsState.toDnsState(): DnsState =
    when (this) {
        ModelDnsState.Default -> DnsState.DEFAULT
        ModelDnsState.Custom -> DnsState.CUSTOM
    }

fun ModelCustomDnsOptions.toCustomOptions(): CustomDnsOptions =
    CustomDnsOptions.newBuilder().addAllAddresses(this.addresses.map { it.toString() }).build()

fun ModelDefaultDnsOptions.toDefaultOptions(): DefaultDnsOptions =
    DefaultDnsOptions.newBuilder()
        .setBlockAds(this.blockAds)
        .setBlockGambling(this.blockGambling)
        .setBlockMalware(this.blockMalware)
        .setBlockTrackers(this.blockTrackers)
        .setBlockAdultContent(this.blockAdultContent)
        .setBlockSocialMedia(this.blockSocialMedia)
        .build()

fun ModelQuantumResistantState.toQuantumResistantState(): QuantumResistantState =
    QuantumResistantState.newBuilder()
        .setState(
            when (this) {
                ModelQuantumResistantState.Auto -> QuantumResistantState.State.AUTO
                ModelQuantumResistantState.On -> QuantumResistantState.State.ON
                ModelQuantumResistantState.Off -> QuantumResistantState.State.OFF
            }
        )
        .build()

fun ModelObfuscationSettings.toObfuscationSettings(): ObfuscationSettings =
    ObfuscationSettings.newBuilder()
        .setSelectedObfuscation(this.selectedObfuscation.toSelectedObfuscation())
        .setUdp2Tcp(this.udp2tcp.toUdp2TcpObfuscationSettings())
        .build()

fun ModelSelectedObfuscation.toSelectedObfuscation(): SelectedObfuscation =
    when (this) {
        ModelSelectedObfuscation.Udp2Tcp -> SelectedObfuscation.UDP2TCP
        ModelSelectedObfuscation.Auto -> SelectedObfuscation.AUTO
        ModelSelectedObfuscation.Off -> SelectedObfuscation.OFF
    }

fun ModelUdp2TcpObfuscationSettings.toUdp2TcpObfuscationSettings(): Udp2TcpObfuscationSettings =
    when (val port = this.port) {
        is Constraint.Any -> Udp2TcpObfuscationSettings.newBuilder().clearPort().build()
        is Constraint.Only ->
            Udp2TcpObfuscationSettings.newBuilder().setPort(port.value.value).build()
    }

fun AppVersionInfo.toModelAppVersionInfo(): ModelAppVersionInfo =
    ModelAppVersionInfo(supported = this.supported, suggestedUpgrade = this.suggestedUpgrade)
