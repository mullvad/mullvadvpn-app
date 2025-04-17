//
//  MullvadConnectionModeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-02-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

// swiftlint:disable:next function_body_length
public func initAccessMethodSettingsWrapper(methods: [PersistentAccessMethod])
    -> SwiftAccessMethodSettingsWrapper {
    // 1. Get all the built in access methods, it is expected that they are always available
    let directMethod = methods.first(where: { $0.proxyConfiguration == .direct })!
    let bridgesMethod = methods.first(where: { $0.proxyConfiguration == .bridges })!
    let encryptedDNSMethod = methods.first(where: { $0.proxyConfiguration == .encryptedDNS })!

    // 2. Get the custom access methods
    let defaultMethods: [PersistentProxyConfiguration] = [.direct, .bridges, .encryptedDNS]
    let customMethods = methods.filter { defaultMethods.contains($0.proxyConfiguration) == false }

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

    var rawCustomMethods = ContiguousArray<UnsafeRawPointer?>([])
    // 4. Convert the custom access methods (all takes different parameters)
    for method in customMethods {
        if case let .shadowsocks(config) = method.proxyConfiguration {
            let serverAddress = config.server.rawValue.map { $0 }
            let shadowsocksConfiguration = new_shadowsocks_access_method_setting(
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
            rawCustomMethods.append(shadowsocksMethodRaw)
        }
        if case let .socks5(config) = method.proxyConfiguration {
            let serverAddress = config.server.rawValue.map { $0 }
            let socks5Configuration = new_socks5_access_method_setting(
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
            rawCustomMethods.append(socks5MethodRaw)
        }
    }

    // 5. Reunite them all in one, and pass it to rust
    return rawCustomMethods.withUnsafeMutableBufferPointer(
        {
            init_access_method_settings_wrapper(
                directMethodRaw,
                bridgesMethodRaw,
                encryptedDNSMethodRaw,
                $0.baseAddress!,
                UInt(customMethods.count)
            )
        }
    )
}
