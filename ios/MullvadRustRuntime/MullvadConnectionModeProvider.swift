//
//  MullvadConnectionModeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-02-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public func initAccessMethodSettingsWrapper(methods: [PersistentAccessMethod]) -> SwiftAccessMethodSettingsContext {
    let validShadowsocksCiphers = ShadowsocksCipherService().getCiphers()

    // 1. Get all the built in access methods, it is expected that they are always available
    let directMethod = methods.first(where: { $0.proxyConfiguration == .direct })!
    let bridgesMethod = methods.first(where: { $0.proxyConfiguration == .bridges })!
    let encryptedDNSMethod = methods.first(where: { $0.proxyConfiguration == .encryptedDNS })!

    // 2. Get the custom access methods
    let defaultMethods: [PersistentProxyConfiguration] = [.direct, .bridges, .encryptedDNS]
    let customMethods = methods.filter {
        // Make sure we only use access methods with valid ciphers.
        if case .shadowsocks(let config) = $0.proxyConfiguration {
            guard validShadowsocksCiphers.contains(config.cipher) else {
                return false
            }
        }

        return !defaultMethods.contains($0.proxyConfiguration)
    }

    // TODO: REMOVE UNWRAPS
    // 3. Convert the builtin access methods
    let directMethodRaw = convertAccessMethod(accessMethod: directMethod)!
    let bridgesMethodRaw = convertAccessMethod(accessMethod: bridgesMethod)!
    let encryptedDNSMethodRaw = convertAccessMethod(accessMethod: encryptedDNSMethod)!

    // 4. Convert the custom access methods (all takes different parameters)
    var rawCustomMethods = customMethods.map { convertAccessMethod(accessMethod: $0)! }

    // 5. Reunite them all in one, and pass it to rust
    let customMethodCount = rawCustomMethods.count
    return initAccessMethodSettingsWrapper(
        direct: directMethodRaw,
        bridges: bridgesMethodRaw,
        encryptedDns: encryptedDNSMethodRaw,
        custom: rawCustomMethods,
    )
}

public func convertAccessMethod(accessMethod: PersistentAccessMethod) -> AccessMethodSettingWrapper? {
    switch accessMethod.proxyConfiguration {
    case .direct, .bridges, .encryptedDNS:
        return convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: accessMethod.kind(),
            proxyConfiguration: nil
        )
    case let .shadowsocks(configuration):
        let serverAddress = configuration.server.rawValue.map { $0 }
        let shadowsocksConfiguration = new_shadowsocks_access_method_setting(
            serverAddress,
            UInt(serverAddress.count),
            configuration.port,
            configuration.password,
            configuration.cipher
        )
        let shadowsocksMethodRaw = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: accessMethod.kind(),
            proxyConfiguration: UInt64(Int(bitPattern: shadowsocksConfiguration))
        )
        return shadowsocksMethodRaw
    case let .socks5(configuration):
        let serverAddress = configuration.server.rawValue.map { $0 }
        let socks5Configuration = new_socks5_access_method_setting(
            serverAddress,
            UInt(serverAddress.count),
            configuration.port,
            configuration.credential?.username,
            configuration.credential?.password
        )
        let socks5MethodRaw = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: accessMethod.kind(),
            proxyConfiguration: UInt64(Int(bitPattern: socks5Configuration))
        )
        return socks5MethodRaw
    }
}

fileprivate
    extension PersistentAccessMethod
{
    func kind() -> SwiftAccessMethodKind {
        switch kind {
        case .direct: .kindDirect
        case .bridges: .kindBridge
        case .encryptedDNS: .kindEncryptedDnsProxy
        case .shadowsocks: .kindShadowsocks
        case .socks5: .kindShadowsocks
        }
    }
}
