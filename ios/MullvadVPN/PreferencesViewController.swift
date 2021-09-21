//
//  PreferencesViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

class PreferencesViewController: UITableViewController, TunnelObserver {

    private let logger = Logger(label: "PreferencesViewController")
    private var dnsSettings: DNSSettings?

    private enum CellIdentifier: String {
        case switchCell
    }

    private let staticDataSource = PreferencesTableViewDataSource()

    init() {
        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.sectionHeaderHeight = UIMetrics.sectionSpacing
        tableView.sectionFooterHeight = 0

        tableView.dataSource = staticDataSource
        tableView.delegate = staticDataSource

        tableView.register(SettingsSwitchCell.self, forCellReuseIdentifier: CellIdentifier.switchCell.rawValue)
        tableView.register(EmptyTableViewHeaderFooterView.self, forHeaderFooterViewReuseIdentifier: EmptyTableViewHeaderFooterView.reuseIdentifier)

        navigationItem.title = NSLocalizedString("NAVIGATION_TITLE", tableName: "Preferences", comment: "Navigation title")
        navigationItem.largeTitleDisplayMode = .always

        TunnelManager.shared.addObserver(self)
        self.dnsSettings = TunnelManager.shared.tunnelInfo?.tunnelSettings.interface.dnsSettings

        setupDataSource()
    }

    // MARK: - TunnelObserver

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelInfo: TunnelInfo?) {
        if tunnelInfo?.tunnelSettings.interface.dnsSettings != self.dnsSettings {
            self.dnsSettings = tunnelInfo?.tunnelSettings.interface.dnsSettings
            self.tableView.reloadData()
        }
    }

    // MARK: - Private

    private func setupDataSource() {
        let blockAdvertisingRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.switchCell.rawValue) { (indexPath, cell) in
            let cell = cell as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString("BLOCK_ADS_CELL_LABEL", tableName: "Preferences", comment: "")
            cell.setOn(self.dnsSettings?.blockAdvertising ?? false, animated: false)
            cell.action = { [weak self] (isOn) in
                self?.dnsSettings?.blockAdvertising = isOn
                self?.saveDNSSettings()
            }
        }
        blockAdvertisingRow.isSelectable = false

        let blockTrackingRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.switchCell.rawValue) { (indexPath, cell) in
            let cell = cell as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString("BLOCK_TRACKERS_CELL_LABEL", tableName: "Preferences", comment: "")
            cell.setOn(self.dnsSettings?.blockTracking ?? false, animated: false)
            cell.action = { [weak self] (isOn) in
                self?.dnsSettings?.blockTracking = isOn
                self?.saveDNSSettings()
            }
        }
        blockTrackingRow.isSelectable = false

        let section = StaticTableViewSection()
        section.addRows([blockAdvertisingRow, blockTrackingRow])
        staticDataSource.addSections([section])
    }

    private func saveDNSSettings() {
        guard let dnsSettings = dnsSettings else { return }

        TunnelManager.shared.setDNSSettings(dnsSettings)
            .onFailure { [weak self] error in
                self?.logger.error(chainedError: error, message: "Failed to save DNS settings")
            }
            .observe { _ in }
    }

}

class PreferencesTableViewDataSource: StaticTableViewDataSource {

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return tableView.dequeueReusableHeaderFooterView(withIdentifier: EmptyTableViewHeaderFooterView.reuseIdentifier)
    }

}
