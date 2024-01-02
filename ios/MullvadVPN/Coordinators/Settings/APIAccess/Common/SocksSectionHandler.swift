//
//  SocksSectionHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

/// Type responsible for handling cells in socks table view section.
struct SocksSectionHandler {
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
        if case .phone = cell.traitCollection.userInterfaceIdiom {
            contentConfiguration.textFieldProperties.keyboardType = .numberPad
        }
        cell.contentConfiguration = contentConfiguration
    }

    private func configureAuthentication(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = SwitchCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.isOn = subject.value.socks.authenticate
        contentConfiguration.onChange = subject.bindSwitchAction(to: \.socks.authenticate)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureUsername(_ cell: UITableViewCell, itemIdentifier: SocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
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
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.socks.password
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.socks.password)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.textFieldProperties.isSecureTextEntry = true
        contentConfiguration.textFieldProperties.textContentType = .password
        cell.contentConfiguration = contentConfiguration
    }
}
