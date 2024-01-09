//
//  AbstractTun.swift
//  PacketTunnel
//
//  Created by Emils on 17/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import CoreFoundation
import Foundation
import Network
import NetworkExtension
import WireGuardKit
import WireGuardKitC
import WireGuardKitTypes

// Wrapper class around AbstractTun to provide an interface similar to WireGuardAdapter.
class AbstractTunAdapter: TunnelAdapterProtocol  {
    private let abstractTun: AbstractTun
    private let queue: DispatchQueue
    
    init(queue: DispatchQueue, packetTunnel: PacketTunnelProvider) {
        self.queue = queue
        abstractTun = AbstractTun(queue: queue, packetTunnel: packetTunnel)
    }
    
    
    func start(configuration: PacketTunnelCore.TunnelAdapterConfiguration) async throws {
        return try await withCheckedThrowingContinuation {
            continuation in
            if case let .failure(error)  = self.start(tunnelConfiguration: configuration) {
                continuation.resume(throwing: error)
            } else {
                continuation.resume(returning: ())
            }
        }
    }
    
    func stop() async throws {
        return await withCheckedContinuation {
            continuation in
            self.stop(completionHandler: {_ in 
                continuation.resume(returning: ())
            }) 
        }
    }

    public func start(tunnelConfiguration: TunnelAdapterConfiguration) -> Result<Void, AbstractTunError> {
        return abstractTun.start(tunnelConfig: tunnelConfiguration)
    }

    public func block(tunnelConfiguration: TunnelConfiguration) -> Result<Void, AbstractTunError> {
        return abstractTun.block(tunnelConfiguration: tunnelConfiguration)
    }

    public func update(tunnelConfiguration: TunnelAdapterConfiguration) -> Result<Void, AbstractTunError> {
        return abstractTun.update(tunnelConfiguration: tunnelConfiguration)
    }

    public func stop(completionHandler: @escaping (WireGuardAdapterError?) -> Void) {
        abstractTun.stop()
        completionHandler(nil)
    }

    public func stats() -> WgStats {
        return abstractTun.stats
    }

    /// Returns the tunnel device interface name, or nil on error.
    /// - Returns: String.
    public var interfaceName: String? {
        guard let tunnelFileDescriptor = self.tunnelFileDescriptor else { return nil }

        var buffer = [UInt8](repeating: 0, count: Int(IFNAMSIZ))

        return buffer.withUnsafeMutableBufferPointer { mutableBufferPointer in
            guard let baseAddress = mutableBufferPointer.baseAddress else { return nil }

            var ifnameSize = socklen_t(IFNAMSIZ)
            let result = getsockopt(
                tunnelFileDescriptor,
                2 /* SYSPROTO_CONTROL */,
                2 /* UTUN_OPT_IFNAME */,
                baseAddress,
                &ifnameSize
            )

            if result == 0 {
                return String(cString: baseAddress)
            } else {
                return nil
            }
        }
    }

    /// Tunnel device file descriptor.
    private var tunnelFileDescriptor: Int32? {
        var ctlInfo = ctl_info()
        withUnsafeMutablePointer(to: &ctlInfo.ctl_name) {
            $0.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: $0.pointee)) {
                _ = strcpy($0, "com.apple.net.utun_control")
            }
        }
        for fd: Int32 in 0 ... 1024 {
            var addr = sockaddr_ctl()
            var ret: Int32 = -1
            var len = socklen_t(MemoryLayout.size(ofValue: addr))
            withUnsafeMutablePointer(to: &addr) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    ret = getpeername(fd, $0, &len)
                }
            }
            if ret != 0 || addr.sc_family != AF_SYSTEM {
                continue
            }
            if ctlInfo.ctl_id == 0 {
                ret = ioctl(fd, CTLIOCGINFO, &ctlInfo)
                if ret != 0 {
                    continue
                }
            }
            if addr.sc_id == ctlInfo.ctl_id {
                return fd
            }
        }
        return nil
    }
}

extension AbstractTunAdapter: TunnelDeviceInfoProtocol {
    func getStats() throws -> PacketTunnelCore.WgStats {
        return self.stats()
    }
}



class AbstractTun: NSObject {
    private var tunRef: OpaquePointer?
    private var dispatchQueue: DispatchQueue

    private let packetTunnelProvider: PacketTunnelProvider

    private var v4SessionMap = [UInt32: NWUDPSession]()
    private var v6SessionMap = [[UInt16]: NWUDPSession]()

    private var v4UdpSession: NWUDPSession?
    private var v6UdpSession: NWUDPSession?

    private let tunQueue = DispatchQueue(label: "AbstractTun", qos: .userInitiated)

    private var wgTaskTimer: DispatchSourceTimer?

    private var socketObservers: [UInt32: NSKeyValueObservation] = [:]

    private(set) var bytesReceived: UInt64 = 0
    private(set) var bytesSent: UInt64 = 0

    var stats: WgStats {
        get {
            return WgStats(bytesReceived: bytesReceived, bytesSent: bytesSent)
        }
    }

    init(queue: DispatchQueue, packetTunnel: PacketTunnelProvider) {
        dispatchQueue = queue
        packetTunnelProvider = packetTunnel
    }

    
    deinit {
        self.stop()
    }

    func stopAbstractTun() {
        abstract_tun_drop(self.tunRef)
        self.tunRef = nil
    }

    func stopOnQueue() {
        dispatchQueue.sync {
            [weak self] in
            self?.stop()
        }
    }

    func stop() {
        wgTaskTimer?.cancel()
        wgTaskTimer = nil
        stopAbstractTun()
    }

    func update(tunnelConfiguration: TunnelAdapterConfiguration) -> Result<Void, AbstractTunError> {
//        dispatchPrecondition(condition: .onQueue(dispatchQueue))
        stop()
        bytesSent = 0
        bytesReceived = 0
        return start(tunnelConfig: tunnelConfiguration)
    }

    func start(tunnelConfig: TunnelAdapterConfiguration) -> Result<Void, AbstractTunError> {
//        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        wgTaskTimer = DispatchSource.makeTimerSource(queue: dispatchQueue)
        wgTaskTimer?.setEventHandler(handler: {
            [weak self] in
            guard let self = self else { return }
            self.handleTimerEvent()
        })
        wgTaskTimer?.schedule(deadline: .now() + .milliseconds(10), repeating: .milliseconds(100))

        let wgConfig = tunnelConfig.asWgConfig
        let singlePeer = wgConfig.peers[0]

        let privateKey = wgConfig.interface.privateKey.rawValue
        guard let peerEndpoint = singlePeer.endpoint else {
            return .failure(AbstractTunError.noPeers)
        }
        let peerAddr = peerEndpoint.host

        var addrBytes = Data(count: 16)
        var addressKind = UInt8(2)
        switch peerAddr {
        case let .ipv4(addr):
            addrBytes[0 ... 3] = addr.rawValue[0 ... 3]
            addressKind = UInt8(AF_INET)
        case let .ipv6(addr):
            addrBytes[0 ... 16] = addr.rawValue[0 ... 16]
            addressKind = UInt8(AF_INET6)
        default:
            break
        }

        var params = IOSTunParams()
        params.peer_addr_version = addressKind
        params.peer_port = singlePeer.endpoint?.port.rawValue ?? UInt16(0)

        withUnsafeMutableBytes(of: &params.peer_key) {
            let _ = singlePeer.publicKey.rawValue.copyBytes(to: $0)
        }

        withUnsafeMutableBytes(of: &params.private_key) {
            let _ = privateKey.copyBytes(to: $0)
        }

        withUnsafeMutableBytes(of: &params.peer_addr_bytes) {
            let _ = addrBytes.copyBytes(to: $0)
        }

        withUnsafePointer(to: params) {
            tunRef = abstract_tun_init_instance($0)
        }
        if tunRef == nil {
            return .failure(AbstractTunError.initializationError)
        }
        packetTunnelProvider.packetFlow.readPackets(completionHandler: { [weak self] data, ipv in
            self?.readPacketTunnelBytes(data, ipversion: ipv)
        })

        self.initializeV4Sockets(peerConfigurations: wgConfig.peers)

        wgTaskTimer?.resume()

        return setConfiguration(wgConfig)
    }

    func setConfiguration(_ config: TunnelConfiguration) -> Result<Void, AbstractTunError> {
        let dispatchGroup = DispatchGroup()
        dispatchGroup.enter()
        var systemError: Error?

        self.packetTunnelProvider
            .setTunnelNetworkSettings(generateNetworkSettings(tunnelConfiguration: config)) { error in
                systemError = error
                dispatchGroup.leave()
            }

        let setNetworkSettingsTimeout = 5
        switch dispatchGroup.wait(wallTimeout: .now() + .seconds(setNetworkSettingsTimeout)) {
        case .success:
            if let error = systemError {
                return .failure(AbstractTunError.setNetworkSettings(error))
            }
            return .success(())
        case .timedOut:
            return .failure(AbstractTunError.setNetworkSettingsTimeout)
        }
    }

    func readPacketTunnelBytes(_ traffic: [Data], ipversion: [NSNumber]) {
        self.dispatchQueue.sync {
            guard let tunPtr = self.tunRef else {
                return
            }
            let receivedDataArr = DataArray(traffic)
            var dataArrPtr = receivedDataArr.toRaw()

            var ioOutput = abstract_tun_handle_host_traffic(tunPtr, dataArrPtr)

            self.handleUdpSendV4(packets: ioOutput.udpV4Traffic())
            ioOutput.discard()
            self.packetTunnelProvider.packetFlow.readPackets(completionHandler: self.readPacketTunnelBytes)
        }
    }

    func receiveTunnelTraffic(_ traffic: [Data]) {
        guard let tunPtr = self.tunRef else {
            return
        }

        let arr = DataArray(traffic)
        var ioOutput = abstract_tun_handle_tunnel_traffic(tunPtr, arr.toRaw())

        let totalDataReceived = traffic.reduce(into: UInt64(0)) { result, current in
            result += UInt64(current.count)
        }
        self.bytesReceived += totalDataReceived

        handleUdpSendV4(packets: ioOutput.udpV4Traffic())
        handleTunSendV4(packets: ioOutput.hostV4Traffic())
        ioOutput.discard()
    }

    func handleTimerEvent() {
        guard let tunPtr = self.tunRef else {
            return
        }

        var ioOutput = abstract_tun_handle_timer_event(tunPtr)
        handleUdpSendV4(packets: ioOutput.udpV4Traffic())
        handleTunSendV4(packets: ioOutput.hostV4Traffic())
        ioOutput.discard()
    }

    // ffiPackets will be invalidated - the inner pointer will be consumed and released.
    private func handleUdpSendV4(
        packets: [Data]
    ) {
        guard !packets.isEmpty else {
            return
        }
        var socket: NWUDPSession
        let dispatchGroup = DispatchGroup()

        if let existingSocket = v4UdpSession {
            socket = existingSocket

            if socket.state == .ready {
                dispatchGroup.enter()
                socket.writeMultipleDatagrams(packets) { error in
                    if let error = error {
                        print(error)
                    }
                    dispatchGroup.leave()
                }
                dispatchGroup.wait()

                let size = packets.reduce(into: 0) { total, packet in total += packet.count }
                bytesSent += UInt64(size)
            }
        }
    }

    private func initializeV4Sockets(peerConfigurations peers: [PeerConfiguration]) {
        var map = [UInt32: NWUDPSession]()
        let dispatchGroup = DispatchGroup()
        var socketObservers: [NSKeyValueObservation] = []

        for peer in peers {
            if let endpoint = peer.endpoint, case let .ipv4(addr) = endpoint.host, endpoint.hasHostAsIPAddress() {
                let endpoint = NetworkExtension.NWHostEndpoint(hostname: "\(endpoint.host)", port: "\(endpoint.port)")

                let session = packetTunnelProvider.createUDPSession(to: endpoint, from: nil)
                let addrBytes = addr.rawValue.withUnsafeBytes { rawPtr in
                    return CFSwapInt32(rawPtr.load(as: UInt32.self))
                }

                let observer = session.observe(\.state, options: [.old, .new]) { session, _ in
                    let newState = session.state
                    switch newState {
                    case .ready:
                        dispatchGroup.leave()
                    default:
                        break
                    }
                }
                if session.state != .ready {
                    dispatchGroup.enter()
                    socketObservers.append(observer)
                } else {
                    observer.invalidate()
                }

                map[addrBytes] = session
                v4UdpSession = session
            }
        }

        // TODO: add timeout here, and error out if the sockets fail to get ready _soon_ enough
        dispatchGroup.wait()
        for observer in socketObservers {
            observer.invalidate()
        }

        v4SessionMap = map
        initializeUdpSessionReadHandlers()
    }

    private func initializeUdpSessionReadHandlers() {
        let readHandler = {
            [weak self] (traffic: [Data]?, error: (any Error)?) in
            guard let self, let traffic else { return }

            self.dispatchQueue.async {
                self.receiveTunnelTraffic(traffic)
            }
        }
        for (_, socket) in self.v4SessionMap {
            socket.setReadHandler(readHandler, maxDatagrams: 2000)
        }

        for (_, socket) in self.v6SessionMap {
            socket.setReadHandler(readHandler, maxDatagrams: 2000)
        }
    }

    private func handleTunSendV4(
        packets: [Data]
    ) {
        if packets.isEmpty {
            return
        }
        let protocols = Array(repeating: NSNumber(value: AF_INET), count: packets.count)
        let totalPacketSize = packets.reduce(into: 0) { total, packet in total += packet.count }
        packetTunnelProvider.packetFlow.writePackets(packets, withProtocols: protocols)

        bytesReceived += UInt64(totalPacketSize)
    }

    func block(tunnelConfiguration: TunnelConfiguration) -> Result<Void, AbstractTunError> {
        return setConfiguration(tunnelConfiguration)
    }
}


func generateNetworkSettings(tunnelConfiguration: TunnelConfiguration) -> NEPacketTunnelNetworkSettings {
    /* iOS requires a tunnel endpoint, whereas in WireGuard it's valid for
     * a tunnel to have no endpoint, or for there to be many endpoints, in
     * which case, displaying a single one in settings doesn't really
     * make sense. So, we fill it in with this placeholder, which is not
     * a valid IP address that will actually route over the Internet.
     */
    let serverEndpoints = tunnelConfiguration.peers.compactMap { $0.endpoint }
        .compactMap { switch $0.host {
        case let .ipv4(addr):
            "\(addr)"
        default:
            nil
        }}

    let endpoint = serverEndpoints.first ?? "127.0.0.1"
    var networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: endpoint)

    if !tunnelConfiguration.interface.dnsSearch.isEmpty || !tunnelConfiguration.interface.dns.isEmpty {
        let dnsServerStrings = tunnelConfiguration.interface.dns.map { $0.stringRepresentation }
        let dnsSettings = NEDNSSettings(servers: dnsServerStrings)
        dnsSettings.searchDomains = tunnelConfiguration.interface.dnsSearch
        if !tunnelConfiguration.interface.dns.isEmpty {
            dnsSettings.matchDomains = [""] // All DNS queries must first go through the tunnel's DNS
        }
        networkSettings.dnsSettings = dnsSettings
    }

    let mtu = tunnelConfiguration.interface.mtu ?? 0

    /* 0 means automatic MTU. In theory, we should just do
     * `networkSettings.tunnelOverheadBytes = 80` but in
     * practice there are too many broken networks out there.
     * Instead set it to 1280. Boohoo. Maybe someday we'll
     * add a nob, maybe, or iOS will do probing for us.
     */
    if mtu == 0 {
        #if os(iOS)
        networkSettings.mtu = NSNumber(value: 1280)
        #elseif os(macOS)
        networkSettings.tunnelOverheadBytes = 80
        #else
        #error("Unimplemented")
        #endif
    } else {
        networkSettings.mtu = NSNumber(value: mtu)
    }

    let (ipv4Addresses, ipv6Addresses) = addresses(tunnelConfiguration: tunnelConfiguration)
    let (ipv4IncludedRoutes, ipv6IncludedRoutes) = includedRoutes(tunnelConfiguration: tunnelConfiguration)

    let ipv4Settings = NEIPv4Settings(
        addresses: ipv4Addresses.map { $0.destinationAddress },
        subnetMasks: ipv4Addresses.map { $0.destinationSubnetMask }
    )

    ipv4Settings.includedRoutes = [
        NEIPv4Route.default(),
        NEIPv4Route(destinationAddress: "10.64.0.1", subnetMask: "255.255.255.255"),
    ]
    // ipv4Settings.excludedRoutes = excludedRoutes
    networkSettings.ipv4Settings = ipv4Settings

    let ipv6Settings = NEIPv6Settings(
        addresses: ipv6Addresses.map { $0.destinationAddress },
        networkPrefixLengths: ipv6Addresses.map { $0.destinationNetworkPrefixLength }
    )
//    ipv6Settings.includedRoutes = ipv6IncludedRoutes
    ipv6Settings.includedRoutes = [NEIPv6Route.default()]
    networkSettings.ipv6Settings = ipv6Settings

    return networkSettings
}

private func addresses(tunnelConfiguration: TunnelConfiguration) -> ([NEIPv4Route], [NEIPv6Route]) {
    var ipv4Routes = [NEIPv4Route]()
    var ipv6Routes = [NEIPv6Route]()
    for addressRange in tunnelConfiguration.interface.addresses {
        if addressRange.address is IPv4Address {
            ipv4Routes.append(NEIPv4Route(
                destinationAddress: "\(addressRange.address)",
                subnetMask: "\(addressRange.subnetMask())"
            ))
        } else if addressRange.address is IPv6Address {
            /* Big fat ugly hack for broken iOS networking stack: the smallest prefix that will have
             * any effect on iOS is a /120, so we clamp everything above to /120. This is potentially
             * very bad, if various network parameters were actually relying on that subnet being
             * intentionally small. TODO: talk about this with upstream iOS devs.
             */
            ipv6Routes.append(NEIPv6Route(
                destinationAddress: "\(addressRange.address)",
                networkPrefixLength: NSNumber(value: min(120, addressRange.networkPrefixLength))
            ))
        }
    }
    return (ipv4Routes, ipv6Routes)
}

private func includedRoutes(tunnelConfiguration: TunnelConfiguration) -> ([NEIPv4Route], [NEIPv6Route]) {
    var ipv4IncludedRoutes = [NEIPv4Route]()
    var ipv6IncludedRoutes = [NEIPv6Route]()

    for addressRange in tunnelConfiguration.interface.addresses {
        if addressRange.address is IPv4Address {
            let route = NEIPv4Route(
                destinationAddress: "\(addressRange.maskedAddress())",
                subnetMask: "\(addressRange.subnetMask())"
            )
            route.gatewayAddress = "\(addressRange.address)"
            ipv4IncludedRoutes.append(route)
        } else if addressRange.address is IPv6Address {
            let route = NEIPv6Route(
                destinationAddress: "\(addressRange.maskedAddress())",
                networkPrefixLength: NSNumber(value: addressRange.networkPrefixLength)
            )
            route.gatewayAddress = "\(addressRange.address)"
            ipv6IncludedRoutes.append(route)
        }
    }

    for peer in tunnelConfiguration.peers {
        for addressRange in peer.allowedIPs {
            if addressRange.address is IPv4Address {
                ipv4IncludedRoutes.append(NEIPv4Route(
                    destinationAddress: "\(addressRange.address)",
                    subnetMask: "\(addressRange.subnetMask())"
                ))
            } else if addressRange.address is IPv6Address {
                ipv6IncludedRoutes.append(NEIPv6Route(
                    destinationAddress: "\(addressRange.address)",
                    networkPrefixLength: NSNumber(value: addressRange.networkPrefixLength)
                ))
            }
        }
    }
    return (ipv4IncludedRoutes, ipv6IncludedRoutes)
}

enum AbstractTunError: Error {
    case initializationError
    case noPeers
    case setNetworkSettings(Error)
    case setNetworkSettingsTimeout
    case noOpenSocket
}

// class UdpSession {
//    private var session: NWUDPSession
//    var ready: Bool
//    var dispatchGroup: DispatchGroup
//
//    init(packetTunnelProvider: PacketTunnelProvider, hostname: String, port: String) {
//
//        let endpoint = NetworkExtension.NWHostEndpoint(hostname: hostname, port: port)
//        session = packetTunnelProvider.createUDPSession(to: endpoint, from: nil)
//
//        ready = session.state == .ready
//    }
//
//    func waitToBeReady() {
//
//    }
//
//    func sendData(data: [Data], completion: ((any Error)?) -> Void) {
//        self.waitToBeReady()
//
//        dispatchGroup.enter()
//        session.writeMultipleDatagrams(data) { [weak self] error in
//            self?.dispatchGroup.leave()
//            completion(error)
//        }
//
//        dispatchGroup.wait()
//    }
//
//    func setReadHandler(maxDatagrams: Int, readHandler: (traffic: [Data]?, error: (any Error)?)) {
//        session.setReadHandler(readHandler, maxDatagrams: maxDatagrams)
//    }
// }
