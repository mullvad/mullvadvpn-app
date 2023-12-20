//
//  ShadowsocksSectionHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

/// Type responsible for handling cells in shadowsocks table view section.
struct ShadowsocksSectionHandler {
    let tableStyle: UITableView.Style
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>

    func configure(_ cell: UITableViewCell, itemIdentifier: ShadowsocksItemIdentifier) {
        switch itemIdentifier {
        case .server:
            configureServer(cell, itemIdentifier: itemIdentifier)
        case .port:
            configurePort(cell, itemIdentifier: itemIdentifier)
        case .password:
            configurePassword(cell, itemIdentifier: itemIdentifier)
        case .cipher:
            configureCipher(cell, itemIdentifier: itemIdentifier)
        }
    }

    func configureServer(_ cell: UITableViewCell, itemIdentifier: ShadowsocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.shadowsocks.server
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.shadowsocks.server)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        cell.contentConfiguration = contentConfiguration
    }

    func configurePort(_ cell: UITableViewCell, itemIdentifier: ShadowsocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.inputText = subject.value.shadowsocks.port
        contentConfiguration.inputFilter = .digitsOnly
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.shadowsocks.port)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        if case .phone = cell.traitCollection.userInterfaceIdiom {
            contentConfiguration.textFieldProperties.keyboardType = .numberPad
        }
        cell.contentConfiguration = contentConfiguration
    }

    func configurePassword(_ cell: UITableViewCell, itemIdentifier: ShadowsocksItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .optional)
        contentConfiguration.inputText = subject.value.shadowsocks.password
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.shadowsocks.password)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.textFieldProperties.isSecureTextEntry = true
        contentConfiguration.textFieldProperties.textContentType = .password
        cell.contentConfiguration = contentConfiguration
    }

    func configureCipher(_ cell: UITableViewCell, itemIdentifier: ShadowsocksItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: tableStyle)
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.secondaryText = subject.value.shadowsocks.cipher.rawValue.description
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
    }
}
