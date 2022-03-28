//
//  SettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsDataSource: NSObject, AccountObserver, UITableViewDataSource, UITableViewDelegate {
    private enum CellReuseIdentifiers: String, CaseIterable {
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
        case main
        case version
        case problemReport
    }

    enum Item: String {
        case account
        case preferences
        case wireguardKey
        case version
        case problemReport
        case faq
    }

    private var snapshot = DataSourceSnapshot<Section, Item>()

    weak var delegate: SettingsDataSourceDelegate?

    weak var tableView: UITableView? {
        didSet {
            tableView?.delegate = self
            tableView?.dataSource = self

            registerClasses()
        }
    }

    override init() {
        super.init()

        Account.shared.addObserver(self)
        updateDataSnapshot()
    }

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { cellIdentifier in
            tableView?.register(cellIdentifier.reusableViewClass, forCellReuseIdentifier: cellIdentifier.rawValue)
        }

        HeaderFooterReuseIdentifier.allCases.forEach { reuseIdentifier in
            tableView?.register(reuseIdentifier.reusableViewClass, forHeaderFooterViewReuseIdentifier: reuseIdentifier.rawValue)
        }
    }

    private func updateDataSnapshot() {
        var newSnapshot = DataSourceSnapshot<Section, Item>()

        if Account.shared.isLoggedIn {
            newSnapshot.appendSections([.main])
            newSnapshot.appendItems([.account, .preferences, .wireguardKey], in: .main)
        }

        newSnapshot.appendSections([.version, .problemReport])
        newSnapshot.appendItems([.version], in: .version)
        newSnapshot.appendItems([.problemReport, .faq], in: .problemReport)

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
        case .account:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.accountCell.rawValue, for: indexPath) as! SettingsAccountCell
            cell.titleLabel.text = NSLocalizedString("ACCOUNT_CELL_LABEL", tableName: "Settings", value: "Account", comment: "")
            cell.accountExpiryDate = Account.shared.expiry
            cell.accessibilityIdentifier = "AccountCell"
            cell.disclosureType = .chevron

            return cell

        case .preferences:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.basicCell.rawValue, for: indexPath) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("PREFERENCES_CELL_LABEL", tableName: "Settings", value: "Preferences", comment: "")
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .chevron

            return cell

        case .wireguardKey:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.basicCell.rawValue, for: indexPath) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("WIREGUARD_KEY_CELL_LABEL", tableName: "Settings", value: "WireGuard key", comment: "")
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = "WireGuardKeyCell"
            cell.disclosureType = .chevron

            return cell

        case .version:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.basicCell.rawValue, for: indexPath) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("APP_VERSION_CELL_LABEL", tableName: "Settings", value: "App version", comment: "")
            cell.detailTitleLabel.text = Bundle.main.productVersion
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .none

            return cell

        case .problemReport:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.basicCell.rawValue, for: indexPath) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("REPORT_PROBLEM_CELL_LABEL", tableName: "Settings", value: "Report a problem", comment: "")
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .chevron

            return cell

        case .faq:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.basicCell.rawValue, for: indexPath) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("FAQ_AND_GUIDES_CELL_LABEL", tableName: "Settings", value: "FAQ & Guides", comment: "")
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .externalLink

            return cell

        }
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        if case .version = snapshot.itemForIndexPath(indexPath) {
            return false
        } else {
            return true
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = snapshot.itemForIndexPath(indexPath) else { return }

        delegate?.settingsDataSource(self, didSelectItem: item)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return tableView.dequeueReusableHeaderFooterView(withIdentifier: HeaderFooterReuseIdentifier.spacer.rawValue)
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

    // MARK: - AccountObserver

    func account(_ account: Account, didUpdateExpiry expiry: Date) {
        tableView?.performBatchUpdates {
            if let indexPath = snapshot.indexPathForItem(.account) {
                tableView?.reloadRows(at: [indexPath], with: .none)
            }
        }
    }

    func account(_ account: Account, didLoginWithToken token: String, expiry: Date) {
        updateDataSnapshot()
        tableView?.reloadData()
    }

    func accountDidLogout(_ account: Account) {
        updateDataSnapshot()
        tableView?.reloadData()
    }
}
