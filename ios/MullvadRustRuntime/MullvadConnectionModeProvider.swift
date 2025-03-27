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

public func initAccessMethodSettingsWrapper(provider: SwiftConnectionModeProviderProxy)
    -> SwiftAccessMethodSettingsWrapper {
    let methods = provider.accessMethods()

    // 1. Get all the built in access methods
    let directMethod = methods.first(where: { $0.proxyConfiguration == .direct })!
    let bridgesMethod = methods.first(where: { $0.proxyConfiguration == .bridges })!
    let encryptedDNSMethod = methods.first(where: { $0.proxyConfiguration == .encryptedDNS })!

    // 2. Get the custom access methods
    let filter: [PersistentProxyConfiguration] = [.direct, .bridges, .encryptedDNS]
    let customMethods = methods.filter { filter.contains($0.proxyConfiguration) == false }

    // 3. Convert the builtin access methods
    let directMethodRaw = convert_builtin_access_method_setting(
        directMethod.id.uuidString,
        directMethod.name,
        directMethod.isEnabled,
        UInt8(KindDirect.rawValue),
        nil
    )
    let bridgesMethodRaw = convert_builtin_access_method_setting(
        bridgesMethod.id.uuidString,
        bridgesMethod.name,
        bridgesMethod.isEnabled,
        UInt8(KindBridge.rawValue),
        nil
    )
    let encryptedDNSMethodRaw = convert_builtin_access_method_setting(
        encryptedDNSMethod.id.uuidString,
        encryptedDNSMethod.name,
        encryptedDNSMethod.isEnabled,
        UInt8(KindEncryptedDnsProxy.rawValue),
        nil
    )

    // 4. Convert the custom access methods (all takes different parameters)
    var customMethodsRaw: ContiguousArray<UnsafeMutableRawPointer?> = []
    for method in customMethods {
        if case let .shadowsocks(config) = method.proxyConfiguration {
            let serverAddress = config.server.rawValue.map { $0 }
            let shadowsocksConfiguration = convert_shadowsocks(
                serverAddress,
                UInt(serverAddress.count),
                config.port,
                config.password,
                config.cipher.rawValue.rawValue
            )
            let shadowsocksMethodRaw = convert_builtin_access_method_setting(
                method.id.uuidString,
                method.name,
                method.isEnabled,
                UInt8(KindShadowsocks.rawValue),
                shadowsocksConfiguration
            )
            customMethodsRaw.append(shadowsocksMethodRaw)
        }
        if case let .socks5(config) = method.proxyConfiguration {
            let serverAddress = config.server.rawValue.map { $0 }
            let socks5Configuration = convert_socks5(
                serverAddress,
                UInt(serverAddress.count),
                config.port,
                config.credential?.username,
                config.credential?.password
            )
            let socks5MethodRaw = convert_builtin_access_method_setting(
                method.id.uuidString,
                method.name,
                method.isEnabled,
                UInt8(KindSocks5Local.rawValue),
                socks5Configuration
            )
            customMethodsRaw.append(socks5MethodRaw)
        }
    }
    // 5. Reunite them all in one, and pass it to rust
    return init_access_method_settings_wrapper(
        directMethodRaw,
        bridgesMethodRaw,
        encryptedDNSMethodRaw,
        customMethodsRaw.map { $0 },
        UInt(customMethodsRaw.count)
    )
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
