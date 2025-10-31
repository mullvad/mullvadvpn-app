//
//  GotaAdapter.swift
//  PacketTunnel
//
//  Created by Emils on 31/10/2025.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import MullvadRustRuntime
import NetworkExtension
import Network
@preconcurrency import WireGuardKitTypes
import PacketTunnelCore

// TODO:
// - NAT64
// - DAITA


public final class GotaAdapter: TunnelAdapterProtocol, Sendable {
    let provider: PacketTunnelProvider
    
    init(provider: PacketTunnelProvider) {
        self.provider = provider
        
    }
    public func start(configuration: PacketTunnelCore.TunnelAdapterConfiguration, daita: WireGuardKitTypes.DaitaConfiguration?) async throws {
        try await self.provider.setTunnelNetworkSettings( generateNetworkSettings(for: configuration))
        
        
    }
    
    public func startMultihop(entryConfiguration: PacketTunnelCore.TunnelAdapterConfiguration?, exitConfiguration: PacketTunnelCore.TunnelAdapterConfiguration, daita: WireGuardKitTypes.DaitaConfiguration?) async throws {
        <#code#>
    }
    
    public func stop() async throws {
        <#code#>
    }
    
    
    private func generateNetworkSettings(for config: TunnelAdapterConfiguration) -> NEPacketTunnelNetworkSettings {
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        
        let dnsServerStrings = config.dns.map { "\($0)" }
        let dnsSettings = NEDNSSettings(servers: dnsServerStrings)
        dnsSettings.matchDomains = [""]
        
        networkSettings.dnsSettings = dnsSettings
        networkSettings.mtu = NSNumber(value: 1280)
        
        let v4Addresses: [NEIPv4Route] = config.interfaceAddresses.compactMap( {address in
            guard address.address is IPv4Address else {
                return nil
            }
            
            return NEIPv4Route(destinationAddress: "\(address.address)", subnetMask: "\(address.subnetMask())")
        } )
        
        networkSettings.ipv4Settings = NEIPv4Settings(addresses: v4Addresses.map{ $0.destinationAddress }, subnetMasks: v4Addresses.map { $0.destinationSubnetMask })
        networkSettings.ipv4Settings?.includedRoutes = [ NEIPv4Route.default()]
        
        let v6Addresses: [NEIPv6Route] = config.interfaceAddresses.compactMap( {address in
            guard address.address is IPv6Address else {
                return nil
            }
            
            return NEIPv6Route(destinationAddress: "\(address.address)", networkPrefixLength: NSNumber(value: min(120, address.networkPrefixLength)))
        } )
        
        
        networkSettings.ipv6Settings = NEIPv6Settings(addresses: v6Addresses.map{ $0.destinationAddress }, networkPrefixLengths: v6Addresses.map { $0.destinationNetworkPrefixLength })
        networkSettings.ipv6Settings?.includedRoutes = [ NEIPv6Route.default()]
        
        return networkSettings
        
        
    }
}





