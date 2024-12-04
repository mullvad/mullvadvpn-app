//
//  SettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

final class SettingsDataSource: UITableViewDiffableDataSource<SettingsDataSource.Section, SettingsDataSource.Item>,
    UITableViewDelegate {
    enum CellReuseIdentifiers: String, CaseIterable {
        case basic

        var reusableViewClass: AnyClass {
            SettingsCell.self
        }
    }

    private enum HeaderFooterReuseIdentifier: String, CaseIterable, HeaderFooterIdentifierProtocol {
        case primary
        case spacer

        var headerFooterClass: AnyClass {
            switch self {
            case .primary:
                UITableViewHeaderFooterView.self
            case .spacer:
                EmptyTableViewHeaderFooterView.self
            }
        }
    }

    enum Section: String {
        case vpnSettings
        case apiAccess
        case version
        case problemReport
    }

    enum Item: String {
        case vpnSettings
        case version
        case problemReport
        case faq
        case apiAccess
        case daita
        case multihop

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .vpnSettings:
                return .vpnSettingsCell
            case .version:
                return .versionCell
            case .problemReport:
                return .problemReportCell
            case .faq:
                return .faqCell
            case .apiAccess:
                return .apiAccessCell
            case .daita:
                return .daitaCell
            case .multihop:
                return .multihopCell
            }
        }

        var reuseIdentifier: CellReuseIdentifiers {
            .basic
        }
    }

    private let interactor: SettingsInteractor
    private var storedAccountData: StoredAccountData?
    private let settingsCellFactory: SettingsCellFactory
    private weak var tableView: UITableView?

    weak var delegate: SettingsDataSourceDelegate?

    init(tableView: UITableView, interactor: SettingsInteractor) {
        self.tableView = tableView
        self.interactor = interactor

        let settingsCellFactory = SettingsCellFactory(tableView: tableView, interactor: interactor)
        self.settingsCellFactory = settingsCellFactory

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            settingsCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.sectionFooterHeight = 0
        tableView.delegate = self
        settingsCellFactory.delegate = self

        registerClasses()
        updateDataSnapshot()

        interactor.didUpdateDeviceState = { [weak self] _ in
            self?.updateDataSnapshot()
        }
        storedAccountData = interactor.deviceState.accountData
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        switch itemIdentifier(for: indexPath) {
        case .vpnSettings, .problemReport, .faq, .apiAccess, .daita, .multihop:
            true
        case .version, .none:
            false
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }
        delegate?.didSelectItem(item: item)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifier.spacer.rawValue
        )
    }

    // MARK: - Private

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { cellIdentifier in
            tableView?.register(
                cellIdentifier.reusableViewClass,
                forCellReuseIdentifier: cellIdentifier.rawValue
            )
        }

        HeaderFooterReuseIdentifier.allCases.forEach { reuseIdentifier in
            tableView?.register(
                reuseIdentifier.headerFooterClass,
                forHeaderFooterViewReuseIdentifier: reuseIdentifier.rawValue
            )
        }
    }

    private func updateDataSnapshot() {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()

        if interactor.deviceState.isLoggedIn {
            snapshot.appendSections([.vpnSettings])
            snapshot.appendItems([
                .daita,
                .multihop,
                .vpnSettings,
            ], toSection: .vpnSettings)
        }

        snapshot.appendSections([.apiAccess])
        snapshot.appendItems([.apiAccess], toSection: .apiAccess)

        snapshot.appendSections([.version, .problemReport])
        snapshot.appendItems([.version], toSection: .version)
        snapshot.appendItems([.problemReport, .faq], toSection: .problemReport)

        apply(snapshot)
    }
}

extension SettingsDataSource: SettingsCellEventHandler {
    func showInfo(for button: SettingsInfoButtonItem) {
        delegate?.showInfo(for: button)
    }

    private func reloadItem(_ item: Item) {
        var snapshot = snapshot()
        snapshot.reloadItems([item])
        apply(snapshot, animatingDifferences: false)
    }
}
