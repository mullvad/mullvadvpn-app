//
//  ShortcutsDataSource.swift
//  MullvadVPN
//
//  Created by Nikolay Davydov on 20.08.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import IntentsUI
import UIKit

final class ShortcutsDataSource: NSObject,
    UITableViewDataSource,
    UITableViewDelegate,
    ShortcutsManagerDelegate
{
    private enum CellReuseIdentifiers: String, CaseIterable {
        case basicCell

        var reusableViewClass: AnyClass {
            switch self {
            case .basicCell:
                return SettingsCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifier: String, CaseIterable {
        case spacer

        var reusableViewClass: AnyClass {
            switch self {
            case .spacer:
                return EmptyTableViewHeaderFooterView.self
            }
        }
    }

    enum Section: String {
        case shortcuts
    }

    struct Item: Hashable {
        let title: String
        let shortcut: INShortcut
        let voiceShortcut: INVoiceShortcut?

        var isAdded: Bool {
            return voiceShortcut != nil
        }
    }

    private var snapshot = DataSourceSnapshot<Section, Item>()

    weak var delegate: ShortcutsDataSourceDelegate?

    weak var tableView: UITableView? {
        didSet {
            tableView?.delegate = self
            tableView?.dataSource = self

            registerClasses()
        }
    }

    override init() {
        super.init()
        updateDataSnapshot(voiceShortcuts: [])
        ShortcutsManager.shared.delegate = self
        ShortcutsManager.shared.updateVoiceShortcuts()
    }

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { cellIdentifier in
            tableView?.register(
                cellIdentifier.reusableViewClass,
                forCellReuseIdentifier: cellIdentifier.rawValue
            )
        }

        HeaderFooterReuseIdentifier.allCases.forEach { reuseIdentifier in
            tableView?.register(
                reuseIdentifier.reusableViewClass,
                forHeaderFooterViewReuseIdentifier: reuseIdentifier.rawValue
            )
        }
    }

    private func updateDataSnapshot(voiceShortcuts: [INVoiceShortcut]) {
        var items = [Item]()

        for data in ShortcutData.allCases {
            guard let shortcut = data.shortcut else { continue }
            let voiceShortcut = voiceShortcuts.first(where: { voiceShortcut in
                isVoiceShortcut(voiceShortcut, invokes: shortcut)
            })
            let item = Item(
                title: data.title,
                shortcut: shortcut,
                voiceShortcut: voiceShortcut
            )
            items.append(item)
        }

        var newSnapshot = DataSourceSnapshot<Section, Item>()
        newSnapshot.appendSections([.shortcuts])
        newSnapshot.appendItems(items, in: .shortcuts)

        snapshot = newSnapshot
    }

    /// Returns whether the voice shortcut performs the same action as the specified shortcut.
    private func isVoiceShortcut(
        _ voiceShortcut: INVoiceShortcut,
        invokes shortcut: INShortcut
    ) -> Bool {
        if let a = voiceShortcut.shortcut.intent, let b = shortcut.intent {
            return type(of: a) == type(of: b)
        }
        return false
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
        let cell = tableView.dequeueReusableCell(
            withIdentifier: CellReuseIdentifiers.basicCell.rawValue,
            for: indexPath
        )
        if let cell = cell as? SettingsCell {
            cell.titleLabel.text = item.title
            cell.disclosureType = item.isAdded ? .tick : .none
        }
        return cell
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = snapshot.itemForIndexPath(indexPath) else { return }
        delegate?.shortcutsDataSource(self, didSelectItem: item)
        tableView.deselectRow(at: indexPath, animated: true)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifier.spacer.rawValue
        )
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        return nil
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        return UIMetrics.sectionSpacing
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        return 0
    }

    // MARK: - ShortcutsManagerDelegate

    func shortcutsManager(
        _ shortcutsManager: ShortcutsManager,
        didReceiveVoiceShortcuts voiceShortcuts: [INVoiceShortcut]
    ) {
        updateDataSnapshot(voiceShortcuts: voiceShortcuts)
        tableView?.reloadData()
    }
}

private extension ShortcutsDataSource {
    enum ShortcutData: CaseIterable {
        case start
        case reconnect
        case stop

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
    }
}
