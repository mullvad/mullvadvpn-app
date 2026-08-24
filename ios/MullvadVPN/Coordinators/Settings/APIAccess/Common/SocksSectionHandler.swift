// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import UIKit

/// Type responsible for handling cells in socks table view section.
@MainActor
struct SocksSectionHandler {
    private let authenticationInputMaxLength = 255

    let tableStyle: UITableView.Style
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>

    func configure(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        switch itemIdentifier {
        case .server:
            configureServer(cell, itemIdentifier: itemIdentifier)
        case .port:
            configurePort(cell, itemIdentifier: itemIdentifier)
        case .username:
            configureUsername(cell, itemIdentifier: itemIdentifier)
        case .password:
            configurePassword(cell, itemIdentifier: itemIdentifier)
        case .authentication:
            configureAuthentication(cell, itemIdentifier: itemIdentifier)
        }
    }

    private func configureServer(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.socks.server
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.socks.server)
        cell.setAccessibilityIdentifier(.socks5ServerCell)
        cell.contentConfiguration = contentConfiguration
    }

    private func configurePort(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.socks.port
        contentConfiguration.inputFilter = .digitsOnly
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.socks.port)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.textFieldProperties.keyboardType = .numberPad
        cell.setAccessibilityIdentifier(.socks5PortCell)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureAuthentication(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = SwitchCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.isOn = subject.value.socks.authenticate
        contentConfiguration.onChange = subject.bindSwitchAction(to: \.socks.authenticate)
        contentConfiguration.accessibilityIdentifier = .socks5AuthenticationSwitch
        cell.contentConfiguration = contentConfiguration
    }

    private func configureUsername(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.maxLength = authenticationInputMaxLength
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.socks.username
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.textFieldProperties.textContentType = .username
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.socks.username)
        cell.contentConfiguration = contentConfiguration
    }

    private func configurePassword(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.maxLength = authenticationInputMaxLength
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.socks.password
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.socks.password)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.textFieldProperties.isSecureTextEntry = true
        contentConfiguration.textFieldProperties.textContentType = .password
        cell.contentConfiguration = contentConfiguration
    }
}
