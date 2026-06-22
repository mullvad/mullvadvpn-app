//
//  AccessMethodViewModel+Persistent.swift
//  MullvadVPN
//
//  Created by pronebird on 15/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import Network

extension AccessMethodViewModel {
    /// Validate view model. Throws on failure.
    ///
    /// - Parameters:
    ///     - shadowsocksCiphers: Supported Shadowsocks ciphers.
    /// - Throws: an instance of ``AccessMethodValidationError``.
    func validate(shadowsocksCiphers: [String] = []) throws {
        _ = try intoPersistentAccessMethod(shadowsocksCiphers: shadowsocksCiphers)
    }

    /// Transform view model into persistent model that can be used with ``AccessMethodRepository``.
    ///
    /// - Parameters:
    ///     - shadowsocksCiphers: Supported Shadowsocks ciphers.
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentAccessMethod``.
    func intoPersistentAccessMethod(shadowsocksCiphers: [String]) throws -> PersistentAccessMethod {
        let configuration: PersistentProxyConfiguration

        do {
            configuration = try intoPersistentProxyConfiguration(shadowsocksCiphers: shadowsocksCiphers)
        } catch let error as AccessMethodValidationError {
            var fieldErrors = error.fieldErrors

            do {
                _ = try validateName()
            } catch let error as AccessMethodValidationError {
                fieldErrors.append(contentsOf: error.fieldErrors)
            }

            throw AccessMethodValidationError(fieldErrors: fieldErrors)
        }

        return PersistentAccessMethod(
            id: id,
            name: try validateName(),
            isEnabled: isEnabled,
            proxyConfiguration: configuration
        )
    }

    /// Transform view model's proxy configuration into persistent configuration that can be used with ``AccessMethodRepository``.
    ///
    /// - Parameters:
    ///     - shadowsocksCiphers: Supported Shadowsocks ciphers.
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentProxyConfiguration``.
    func intoPersistentProxyConfiguration(shadowsocksCiphers: [String]) throws -> PersistentProxyConfiguration {
        switch method {
        case .direct:
            .direct
        case .bridges:
            .bridges
        case .encryptedDNS:
            .encryptedDNS
        case .domainFronting:
            .domainFronting
        case .socks5:
            try socks.intoPersistentProxyConfiguration()
        case .shadowsocks:
            try shadowsocks.intoPersistentProxyConfiguration(shadowsocksCiphers: shadowsocksCiphers)
        }
    }

    private func validateName() throws -> String {
        // Context doesn't matter for name field errors.
        if name.isEmpty {
            let fieldError = AccessMethodFieldValidationError(kind: .emptyValue, field: .name, context: .shadowsocks)
            throw AccessMethodValidationError(fieldErrors: [fieldError])
        } else if name.count > NameInputFormatter.maxLength {
            let fieldError = AccessMethodFieldValidationError(kind: .nameTooLong, field: .name, context: .shadowsocks)
            throw AccessMethodValidationError(fieldErrors: [fieldError])
        }

        return name
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

        if server.isEmpty {
            fieldErrors.append(AccessMethodFieldValidationError(kind: .emptyValue, field: .server, context: context))
        } else {
            switch CommonValidators.parseIPAddress(from: server, context: context) {
            case let .success(serverAddress):
                draftConfiguration.server = serverAddress
            case let .failure(error):
                fieldErrors.append(error)
            }
        }

        if port.isEmpty {
            fieldErrors.append(AccessMethodFieldValidationError(kind: .emptyValue, field: .port, context: context))
        } else {
            switch CommonValidators.parsePort(from: port, context: context) {
            case let .success(port):
                draftConfiguration.port = port
            case let .failure(error):
                fieldErrors.append(error)
            }
        }

        if authenticate {
            if username.isEmpty {
                fieldErrors.append(
                    AccessMethodFieldValidationError(kind: .emptyValue, field: .username, context: context)
                )
            }

            if password.isEmpty {
                fieldErrors.append(
                    AccessMethodFieldValidationError(kind: .emptyValue, field: .password, context: context)
                )
            }

            if !(username.isEmpty && password.isEmpty) {
                draftConfiguration.authentication = .authentication(
                    PersistentProxyConfiguration.UserCredential(
                        username: username,
                        password: password
                    ))
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
    /// - Parameters:
    ///     - shadowsocksCiphers: Supported Shadowsocks ciphers.
    /// - Throws: an instance of ``AccessMethodValidationError``.
    /// - Returns: an instance of ``PersistentProxyConfiguration``.
    func intoPersistentProxyConfiguration(shadowsocksCiphers: [String]) throws -> PersistentProxyConfiguration {
        var draftConfiguration = PersistentProxyConfiguration.ShadowsocksConfiguration(
            server: .ipv4(.loopback),
            port: 0,
            password: "",
            cipher: ""
        )

        let context: AccessMethodFieldValidationError.Context = .shadowsocks
        var fieldErrors: [AccessMethodFieldValidationError] = []

        if server.isEmpty {
            fieldErrors.append(AccessMethodFieldValidationError(kind: .emptyValue, field: .server, context: context))
        } else {
            switch CommonValidators.parseIPAddress(from: server, context: context) {
            case let .success(serverAddress):
                draftConfiguration.server = serverAddress
            case let .failure(error):
                fieldErrors.append(error)
            }
        }

        if port.isEmpty {
            fieldErrors.append(AccessMethodFieldValidationError(kind: .emptyValue, field: .port, context: context))
        } else {
            switch CommonValidators.parsePort(from: port, context: context) {
            case let .success(port):
                draftConfiguration.port = port
            case let .failure(error):
                fieldErrors.append(error)
            }
        }

        draftConfiguration.password = password

        if shadowsocksCiphers.contains(cipher) {
            draftConfiguration.cipher = cipher
        } else {
            fieldErrors.append(AccessMethodFieldValidationError(kind: .invalidCipher, field: .cipher, context: context))
        }

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
        -> Result<UInt16, AccessMethodFieldValidationError>
    {
        guard let portNumber = UInt16(value) else {
            return .failure(AccessMethodFieldValidationError(kind: .invalidPort, field: .port, context: context))
        }

        guard portNumber > 0 else {
            return .failure(AccessMethodFieldValidationError(kind: .invalidPort, field: .port, context: context))
        }

        return .success(portNumber)
    }

    /// Parse IP address from string by using Apple's facilities.
    ///
    /// - Parameters:
    ///   - value: a string input.
    ///   - context: an input context
    /// - Returns: a result containing an IP address on success, otherwise an instance of ``AccessMethodFieldValidationError``.
    static func parseIPAddress(
        from value: String,
        context: AccessMethodFieldValidationError.Context
    ) -> Result<AnyIPAddress, AccessMethodFieldValidationError> {
        if let address = AnyIPAddress(value) {
            return .success(address)
        } else {
            return .failure(AccessMethodFieldValidationError(kind: .invalidIPAddress, field: .server, context: context))
        }
    }
}
