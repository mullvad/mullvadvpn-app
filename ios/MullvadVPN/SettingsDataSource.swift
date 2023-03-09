//
//  SettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class SettingsDataSource: UITableViewDiffableDataSource<
    SettingsDataSource.Section,
    SettingsDataSource.Item
>, UITableViewDelegate {
    enum CellReuseIdentifiers: String, CaseIterable {
        case accountCell
        case basicCell

        var reusableViewClass: AnyClass {
            switch self {
            case .accountCell:
                return SettingsAccountCell.self
            case .basicCell:
                return SettingsCell.self
            }
        }
    }

    enum HeaderFooterReuseIdentifier: String, CaseIterable {
        case spacer

        var reusableViewClass: AnyClass {
            switch self {
            case .spacer:
                return EmptyTableViewHeaderFooterView.self
            }
        }
    }

    enum Section: String {
        case main
        case version
        case problemReport
    }

    enum Item: String {
        case account
        case preferences
        case version
        case problemReport
        case faq
    }

    private let interactor: SettingsInteractor
    private var storedAccountData: StoredAccountData?

    weak var delegate: SettingsDataSourceDelegate?
    weak var tableView: UITableView?

    init(
        tableView: UITableView,
        interactor: SettingsInteractor,
        cellProvider: @escaping UITableViewDiffableDataSource<Section, Item>.CellProvider
    ) {
        self.tableView = tableView
        self.interactor = interactor

        super.init(tableView: tableView, cellProvider: cellProvider)

        tableView.delegate = self
        registerClasses()
        updateDataSnapshot()

        interactor.didUpdateDeviceState = { [weak self] deviceState in
            self?.didUpdateDeviceState(deviceState)
        }
        storedAccountData = interactor.deviceState.accountData
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

    private func updateDataSnapshot() {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()

        if interactor.deviceState.isLoggedIn {
            snapshot.appendSections([.main])
            snapshot.appendItems([.account, .preferences], toSection: .main)
        }

        snapshot.appendSections([.version, .problemReport])
        snapshot.appendItems([.version], toSection: .version)
        snapshot.appendItems([.problemReport, .faq], toSection: .problemReport)

        apply(snapshot)
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        if case .version = snapshot().itemForIndexPath(indexPath) {
            return false
        } else {
            return true
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = snapshot().itemForIndexPath(indexPath) else { return }

        delegate?.settingsDataSource(self, didSelectItem: item)
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

    // MARK: - Private

    private func didUpdateDeviceState(_ deviceState: DeviceState) {
        let newAccountData = deviceState.accountData
        let oldAccountData = storedAccountData

        storedAccountData = newAccountData

        // Refresh individual row if expiry changed.
        if let newAccountData = newAccountData, let oldAccountData = oldAccountData,
           oldAccountData.number == newAccountData.number,
           oldAccountData.expiry != newAccountData.expiry
        {
            var snapshot = snapshot()
            snapshot.reloadItems([.account])
            apply(snapshot)
        }
    }
}
