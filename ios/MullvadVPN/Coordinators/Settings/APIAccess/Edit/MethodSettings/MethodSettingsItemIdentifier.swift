//
//  MethodSettingsItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum MethodSettingsItemIdentifier: Hashable {
    case name
    case `protocol`
    case proxyConfiguration(ProxyProtocolConfigurationItemIdentifier)
    case validationError
    case testingStatus
    case cancelTest

    /// Returns all shadowsocks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static var allShadowsocksItems: [MethodSettingsItemIdentifier] {
        ShadowsocksItemIdentifier.allCases.map { .proxyConfiguration(.shadowsocks($0)) }
    }

    /// Returns all socks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static func allSocksItems(authenticate: Bool) -> [MethodSettingsItemIdentifier] {
        SocksItemIdentifier.allCases(authenticate: authenticate).map { .proxyConfiguration(.socks($0)) }
    }

    /// Cell identifiers for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .name:
            .textInput
        case .protocol:
            .textWithDisclosure
        case let .proxyConfiguration(itemIdentifier):
            itemIdentifier.cellIdentifier
        case .validationError:
            .validationError
        case .testingStatus:
            .testingStatus
        case .cancelTest:
            .button
        }
    }

    /// Returns `true` if the cell background should be made transparent.
    var isClearBackground: Bool {
        switch self {
        case .validationError, .cancelTest, .testingStatus:
            return true
        case .name, .protocol, .proxyConfiguration:
            return false
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case .name, .validationError, .testingStatus, .cancelTest:
            false
        case .protocol:
            true
        case let .proxyConfiguration(itemIdentifier):
            itemIdentifier.isSelectable
        }
    }

    /// The text label for the corresponding cell.
    var text: String? {
        switch self {
        case .name:
            NSLocalizedString("NAME", tableName: "APIAccess", value: "Name", comment: "")
        case .protocol:
            NSLocalizedString("TYPE", tableName: "APIAccess", value: "Type", comment: "")
        case .proxyConfiguration, .validationError:
            nil
        case .cancelTest:
            NSLocalizedString("CANCEL_TEST", tableName: "APIAccess", value: "Cancel", comment: "")
        case .testingStatus:
            nil
        }
    }

    static func fromFieldValidationErrors(
        _ errors: [AccessMethodFieldValidationError],
        selectedMethod: AccessMethodKind
    ) -> [MethodSettingsItemIdentifier] {
        switch selectedMethod {
        case .direct, .bridges:
            []
        case .shadowsocks:
            errors.compactMap { error in
                switch error.field {
                case .server: .proxyConfiguration(.shadowsocks(.server))
                case .port: .proxyConfiguration(.shadowsocks(.port))
                case .username: nil
                case .password: .proxyConfiguration(.shadowsocks(.password))
                }
            }
        case .socks5:
            errors.map { error in
                switch error.field {
                case .server: .proxyConfiguration(.socks(.server))
                case .port: .proxyConfiguration(.socks(.port))
                case .username: .proxyConfiguration(.socks(.username))
                case .password: .proxyConfiguration(.socks(.password))
                }
            }
        }
    }
}
