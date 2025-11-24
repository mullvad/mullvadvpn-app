//
//  SettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

final class SettingsDataSource: UITableViewDiffableDataSource<SettingsDataSource.Section, SettingsDataSource.Item>,
    UITableViewDelegate
{
    enum CellReuseIdentifier: String, CaseIterable {
        case basic
        case changelog

        var reusableViewClass: AnyClass {
            SettingsCell.self
        }

        var cellStyle: UITableViewCell.CellStyle {
            switch self {
            case .basic: .default
            case .changelog: .subtitle
            }
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
        case language
    }

    enum Item: String {
        case vpnSettings
        case changelog
        case problemReport
        case faq
        case apiAccess
        case daita
        case multihop
        case language

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .vpnSettings:
                .vpnSettingsCell
            case .changelog:
                .versionCell
            case .problemReport:
                .problemReportCell
            case .faq:
                .faqCell
            case .apiAccess:
                .apiAccessCell
            case .daita:
                .daitaCell
            case .multihop:
                .multihopCell
            case .language:
                .languageCell
            }
        }

        var reuseIdentifier: CellReuseIdentifier {
            switch self {
            case .changelog: .changelog
            default: .basic
            }
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
        tableView.tableHeaderView = UIView(
            frame: CGRect(
                origin: .zero,
                size: CGSize(width: 0, height: UIMetrics.TableView.emptyHeaderHeight)
            ))
        tableView.delegate = self

        registerClasses()
        updateDataSnapshot()

        interactor.didUpdateSettings = { [weak self] in
            self?.updateDataSnapshot()
        }
        storedAccountData = interactor.deviceState.accountData
    }

    func reload() {
        settingsCellFactory.viewModel = SettingsViewModel(from: interactor.tunnelSettings)

        var snapshot = snapshot()
        snapshot.reconfigureItems(snapshot.itemIdentifiers)
        apply(snapshot, animatingDifferences: false)
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        true
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

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        guard let section = sectionIdentifier(for: section) else { return 0 }

        return switch section {
        case .vpnSettings:
            0
        default:
            UIMetrics.TableView.sectionSpacing
        }
    }

    // MARK: - Private

    private func registerClasses() {
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
            snapshot.appendItems(
                [
                    .daita,
                    .multihop,
                    .vpnSettings,
                ], toSection: .vpnSettings)
        }

        snapshot.appendSections([.language])
        snapshot.appendItems([.language], toSection: .language)

        snapshot.appendSections([.apiAccess])
        snapshot.appendItems([.apiAccess], toSection: .apiAccess)

        snapshot.appendSections([.version, .problemReport])
        snapshot.appendItems([.changelog], toSection: .version)
        snapshot.appendItems([.problemReport, .faq], toSection: .problemReport)

        apply(snapshot)
    }
}
