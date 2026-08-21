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
    let direct = convertAccessMethod(accessMethod: directMethod)!
    let bridges = convertAccessMethod(accessMethod: bridgesMethod)!
    let encryptedDNS = convertAccessMethod(accessMethod: encryptedDNSMethod)!

    // 4. Convert the custom access methods (all takes different parameters)
    let convertedCustomMethods = customMethods.map { convertAccessMethod(accessMethod: $0)! }

    // 5. Reunite them all in one, and pass it to rust
    return initAccessMethodSettingsWrapper(
        direct: direct,
        bridges: bridges,
        encryptedDns: encryptedDNS,
        custom: convertedCustomMethods,
    )
}

public func convertAccessMethod(accessMethod: PersistentAccessMethod) -> AccessMethodSettingWrapper? {
    switch accessMethod.proxyConfiguration {
    // Simple
    case .direct, .bridges, .encryptedDNS:
        return convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: accessMethod.kind(),
        )
    // Not so simple
    case let .shadowsocks(configuration):
        let serverAddress = configuration.server.rawValue
        let shadowsocksConfiguration = newShadowsocksAccessMethodSetting(
            address: serverAddress,
            port: configuration.port,
            password: configuration.password,
            cipher: configuration.cipher
        )
        let shadowsocksMethodRaw = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: .kindShadowsocks(shadowsocksConfiguration),
        )
        return shadowsocksMethodRaw
    // Not so simple
    case let .socks5(configuration):
        let serverAddress = configuration.server.rawValue
        let socks5Configuration = newSocks5AccessMethodSetting(
            address: serverAddress,
            port: configuration.port,
            username: configuration.credential?.username,
            password: configuration.credential?.password
        )
        let socks5MethodRaw = convertBuiltinAccessMethodSetting(
            uniqueIdentifier: accessMethod.id.uuidString,
            name: accessMethod.name,
            isEnabled: accessMethod.isEnabled,
            methodKind: .kindSocks5Local(socks5Configuration),
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
        case _: fatalError()
        }
    }
}
