//
//  SettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
        case misc
        case general
        case problemReport
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
        case notificationSettings

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
            case .notificationSettings:
                .notificationSettingsCell
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
        tableView.deselectRow(at: indexPath, animated: false)
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

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        guard let section = sectionIdentifier(for: section), section == .misc else {
            return nil
        }

        let footerView = tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifier.primary.rawValue
        )

        var contentConfiguration = ListCellContentConfiguration(
            textProperties:
                ListCellContentConfiguration.TextProperties(
                    font: .mullvadTiny,
                    color: .TableSection.footerTextColor
                ),
            directionalLayoutMargins: NSDirectionalEdgeInsets(UIMetrics.SettingsRowView.footerLayoutMargins)
        )
        contentConfiguration.text = NSLocalizedString(
            "Changing language will disconnect you from the VPN and restart the app.",
            comment: ""
        )
        footerView?.contentConfiguration = contentConfiguration

        return footerView
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        guard let section = sectionIdentifier(for: section) else { return 0 }

        return switch section {
        case .misc:
            UITableView.automaticDimension
        default:
            0
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
        let isLoggedIn = interactor.deviceState.isLoggedIn
        if isLoggedIn {
            snapshot.appendSections([.vpnSettings])
            snapshot.appendItems(
                [
                    .daita,
                    .multihop,
                    .vpnSettings,
                ], toSection: .vpnSettings)
        }

        snapshot.appendSections([.apiAccess])
        snapshot.appendItems([.apiAccess], toSection: .apiAccess)

        snapshot.appendSections([.general])
        snapshot.appendItems([.notificationSettings, .changelog], toSection: .general)

        snapshot.appendSections([.misc])
        snapshot.appendItems([.problemReport, .faq, .language], toSection: .misc)

        apply(snapshot)
    }
}
