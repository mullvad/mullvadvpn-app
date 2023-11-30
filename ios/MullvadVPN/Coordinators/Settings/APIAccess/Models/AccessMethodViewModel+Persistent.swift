//
//  AccessMethodViewModel+Persistent.swift
//  MullvadVPN
//
//  Created by pronebird on 15/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

extension AccessMethodViewModel {
    /// Validate view model. Throws on failure.
    ///
    /// - Throws: an instance of ``AccessMethodValidationError``.
    func validate() throws {
        _ = try intoPersistentAccessMethod()
    }

    /// Transform view model into persistent model that can be used with ``AccessMethodRepository``.
    ///
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentAccessMethod``.
    func intoPersistentAccessMethod() throws -> PersistentAccessMethod {
        return PersistentAccessMethod(
            id: id,
            name: name,
            isEnabled: isEnabled,
            proxyConfiguration: try intoPersistentProxyConfiguration()
        )
    }

    /// Transform view model's proxy configuration into persistent configuration that can be used with ``AccessMethodRepository``.
    ///
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentProxyConfiguration``.
    func intoPersistentProxyConfiguration() throws -> PersistentProxyConfiguration {
        switch method {
        case .direct:
            .direct
        case .bridges:
            .bridges
        case .socks5:
            try socks.intoPersistentProxyConfiguration()
        case .shadowsocks:
            try shadowsocks.intoPersistentProxyConfiguration()
        }
    }
}

extension AccessMethodViewModel.Socks {
    /// Transform socks view model into persistent proxy configuration that can be used with ``AccessMethodRepository``.
    ///
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentProxyConfiguration``.
    func intoPersistentProxyConfiguration() throws -> PersistentProxyConfiguration {
        var draftConfiguration = PersistentProxyConfiguration.SocksConfiguration(
            server: .ipv4(.loopback),
            port: 0,
            authentication: .noAuthentication
        )

        let context: AccessMethodFieldValidationError.Context = .socks
        var fieldErrors: [AccessMethodFieldValidationError] = []

        switch CommonValidators.parseIPAddress(from: server, context: context) {
        case let .success(serverAddress):
            draftConfiguration.server = serverAddress
        case let .failure(error):
            fieldErrors.append(error)
        }

        switch CommonValidators.parsePort(from: port, context: context) {
        case let .success(port):
            draftConfiguration.port = port
        case let .failure(error):
            fieldErrors.append(error)
        }

        if authenticate {
            if username.isEmpty {
                fieldErrors.append(AccessMethodFieldValidationError(
                    kind: .emptyValue,
                    field: .username,
                    context: context
                ))
            } else {
                draftConfiguration.authentication = .usernamePassword(username: username, password: password)
            }
        }

        if fieldErrors.isEmpty {
            return .socks5(draftConfiguration)
        } else {
            throw AccessMethodValidationError(fieldErrors: fieldErrors)
        }
    }
}

extension AccessMethodViewModel.Shadowsocks {
    /// Transform shadowsocks view model into persistent proxy configuration that can be used with ``AccessMethodRepository``.
    ///
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentProxyConfiguration``.
    func intoPersistentProxyConfiguration() throws -> PersistentProxyConfiguration {
        var draftConfiguration = PersistentProxyConfiguration.ShadowsocksConfiguration(
            server: .ipv4(.loopback),
            port: 0,
            password: "",
            cipher: .default
        )

        let context: AccessMethodFieldValidationError.Context = .shadowsocks
        var fieldErrors: [AccessMethodFieldValidationError] = []

        switch CommonValidators.parseIPAddress(from: server, context: context) {
        case let .success(serverAddress):
            draftConfiguration.server = serverAddress
        case let .failure(error):
            fieldErrors.append(error)
        }

        switch CommonValidators.parsePort(from: port, context: context) {
        case let .success(port):
            draftConfiguration.port = port
        case let .failure(error):
            fieldErrors.append(error)
        }

        draftConfiguration.cipher = cipher
        draftConfiguration.password = password

        if fieldErrors.isEmpty {
            return .shadowsocks(draftConfiguration)
        } else {
            throw AccessMethodValidationError(fieldErrors: fieldErrors)
        }
    }
}

private enum CommonValidators {
    /// Parse port from string.
    ///
    /// - Parameters:
    ///   - value: a string input.
    ///   - context: an input context.
    /// - Returns: a result containing a parsed port number on success, otherwise an instance of ``AccessMethodFieldValidationError``.
    static func parsePort(from value: String, context: AccessMethodFieldValidationError.Context)
        -> Result<UInt16, AccessMethodFieldValidationError> {
        guard let portNumber = UInt16(value) else {
            return .failure(AccessMethodFieldValidationError(kind: .parsePort, field: .port, context: context))
        }

        guard portNumber > 0 else {
            return .failure(AccessMethodFieldValidationError(kind: .invalidPort, field: .port, context: context))
        }

        return .success(portNumber)
    }

    /// Parse IP address from string by first running the input via regular expression before parsing it using Apple's facilities which are known to accept all kind of
    /// malformed input.
    ///
    /// - Parameters:
    ///   - value: a string input.
    ///   - context: an input context
    /// - Returns: a result containing an IP address on success, otherwise an instance of ``AccessMethodFieldValidationError``.
    static func parseIPAddress(
        from value: String,
        context: AccessMethodFieldValidationError.Context
    ) -> Result<AnyIPAddress, AccessMethodFieldValidationError> {
        let range = NSRange(value.startIndex ..< value.endIndex, in: value)

        let regexMatch = NSRegularExpression.ipv4RegularExpression.firstMatch(in: value, range: range)
            ?? NSRegularExpression.ipv6RegularExpression.firstMatch(in: value, range: range)

        if regexMatch?.range == range, let address = AnyIPAddress(value) {
            return .success(address)
        } else {
            return .failure(AccessMethodFieldValidationError(kind: .parseIPAddress, field: .server, context: context))
        }
    }
}
