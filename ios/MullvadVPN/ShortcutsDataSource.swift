//
//  ShortcutsDataSource.swift
//  MullvadVPN
//
//  Created by Nikolay Davydov on 20.08.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import IntentsUI
import UIKit

final class ShortcutsDataSource: NSObject, UITableViewDataSource, UITableViewDelegate {
    private enum CellReuseIdentifiers: String, CaseIterable {
        case basicCell

        var reusableViewClass: AnyClass {
            switch self {
            case .basicCell:
                return SettingsCell.self
            }
        }
    }

    enum Section: String {
        case shortcuts
    }

    enum Item: String, CaseIterable {
        case start
        case reconnect
        case stop
    }

    private var snapshot = DataSourceSnapshot<Section, Item>()

    weak var delegate: ShortcutsDataSourceDelegate?

    func configure(_ tableView: UITableView) {
        CellReuseIdentifiers.allCases.forEach { cellIdentifier in
            tableView.register(
                cellIdentifier.reusableViewClass,
                forCellReuseIdentifier: cellIdentifier.rawValue
            )
        }
        tableView.dataSource = self
        tableView.delegate = self
    }

    override init() {
        super.init()
        updateDataSnapshot()
    }

    private func updateDataSnapshot() {
        var newSnapshot = DataSourceSnapshot<Section, Item>()
        newSnapshot.appendSections([.shortcuts])
        newSnapshot.appendItems(Item.allCases, in: .shortcuts)
        snapshot = newSnapshot
    }

    // MARK: - UITableViewDataSource

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        let sectionIdentifier = snapshot.section(at: section)!
        return snapshot.numberOfItems(in: sectionIdentifier) ?? 0
    }

    func numberOfSections(in tableView: UITableView) -> Int {
        return snapshot.numberOfSections()
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = snapshot.itemForIndexPath(indexPath)!
        switch item {
        case .start, .reconnect, .stop:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            )
            if let cell = cell as? SettingsCell {
                cell.titleLabel.text = item.title
            }
            return cell
        }
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = snapshot.itemForIndexPath(indexPath) else { return }
        delegate?.shortcutsDataSource(self, didSelectItem: item)
        tableView.deselectRow(at: indexPath, animated: true)
    }
}

extension ShortcutsDataSource.Item {
    var title: String {
        switch self {
        case .start:
            return NSLocalizedString(
                "SHORTCUTS_NAME_START_VPN",
                tableName: "Shortcuts",
                value: "Start VPN",
                comment: ""
            )
        case .reconnect:
            return NSLocalizedString(
                "SHORTCUTS_NAME_RECONNECT_VPN",
                tableName: "Shortcuts",
                value: "Reconnect VPN",
                comment: ""
            )
        case .stop:
            return NSLocalizedString(
                "SHORTCUTS_NAME_STOP_VPN",
                tableName: "Shortcuts",
                value: "Stop VPN",
                comment: ""
            )
        }
    }

    var shortcut: INShortcut? {
        let intent: INIntent
        switch self {
        case .start:
            intent = StartVPNIntent()
        case .reconnect:
            intent = ReconnectVPNIntent()
        case .stop:
            intent = StopVPNIntent()
        }
        intent.suggestedInvocationPhrase = title
        guard let shortcut = INShortcut(intent: intent) else {
            assertionFailure("The shortcut has an invalid intent.")
            return nil
        }
        return shortcut
    }

    init?(voiceShortcut: INVoiceShortcut) {
        switch voiceShortcut.shortcut.intent {
        case is StartVPNIntent:
            self = .start
        case is ReconnectVPNIntent:
            self = .reconnect
        case is StopVPNIntent:
            self = .stop
        default:
            return nil
        }
    }
}
