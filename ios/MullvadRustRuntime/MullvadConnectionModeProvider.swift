//
//  MullvadConnectionModeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-02-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public func initConnectionModeProvider(provider: SwiftConnectionModeProviderProxy) -> SwiftConnectionModeProvider {
    let rawProvider = Unmanaged.passUnretained(provider)
        .toOpaque()
    return init_connection_mode_provider(rawProvider, provider.domainName)
}

@_cdecl("connection_mode_provider_initial")
func connectionModeProviderInitial(rawPointer: UnsafeMutableRawPointer) {
    let accessMethodIterator = Unmanaged<SwiftConnectionModeProviderProxy>
        .fromOpaque(rawPointer)
        .takeUnretainedValue()
    accessMethodIterator.initial()
}

@_cdecl("connection_mode_provider_receive")
func connectionModeProviderReceive(rawIterator: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    let accessMethodIterator = Unmanaged<SwiftConnectionModeProviderProxy>
        .fromOpaque(rawIterator)
        .takeUnretainedValue()
    let proxyConfiguration = accessMethodIterator.pickMethod()

    switch proxyConfiguration {
    case .direct:
        return convert_direct()
    case let .shadowsocks(configuration):
        let serverAddress = configuration.server.rawValue.map { $0 }
        return convert_shadowsocks(
            serverAddress,
            UInt(serverAddress.count),
            configuration.port,
            configuration.password,
            configuration.cipher.rawValue.rawValue
        )
    default:
        break
    }

    return nil
    // TODO: Return something here
}

@_cdecl("connection_mode_provider_rotate")
func connectionModeProviderRotate(rawPointer: UnsafeMutableRawPointer) {
    let accessMethodIterator = Unmanaged<SwiftConnectionModeProviderProxy>
        .fromOpaque(rawPointer)
        .takeRetainedValue()
    accessMethodIterator.rotate()
}
