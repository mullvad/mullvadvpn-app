//
//  MethodSettingsDataSourceConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

class MethodSettingsDataSourceConfiguration {
    private let dataSource: UITableViewDiffableDataSource<
        MethodSettingsSectionIdentifier,
        MethodSettingsItemIdentifier
    >?

    init(
        dataSource: UITableViewDiffableDataSource<MethodSettingsSectionIdentifier, MethodSettingsItemIdentifier>
    ) {
        self.dataSource = dataSource
    }

    func updateDataSource(
        previousValue: AccessMethodViewModel?,
        newValue: AccessMethodViewModel,
        previousValidationError: [AccessMethodFieldValidationError],
        newValidationError: [AccessMethodFieldValidationError],
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<MethodSettingsSectionIdentifier, MethodSettingsItemIdentifier>()

        // Add name field for user-defined access methods.
        if !newValue.method.isPermanent {
            snapshot.appendSections([.name])
            snapshot.appendItems([.name], toSection: .name)
        }

        snapshot.appendSections([.protocol])
        snapshot.appendItems([.protocol], toSection: .protocol)
        // Reconfigure protocol cell on change.
        if let previousValue, previousValue.method != newValue.method {
            snapshot.reconfigureOrReloadItems([.protocol])
        }

        // Add proxy configuration section if the access method is configurable.
        if newValue.method.hasProxyConfiguration {
            snapshot.appendSections([.proxyConfiguration])
        }

        switch newValue.method {
        case .direct, .bridges:
            break

        case .shadowsocks:
            snapshot.appendItems(MethodSettingsItemIdentifier.allShadowsocksItems, toSection: .proxyConfiguration)
            // Reconfigure cipher cell on change.
            if let previousValue, previousValue.shadowsocks.cipher != newValue.shadowsocks.cipher {
                snapshot.reconfigureOrReloadItems([.proxyConfiguration(.shadowsocks(.cipher))])
            }

            // Reconfigure the proxy configuration cell if validation error changed.
            if previousValidationError != newValidationError {
                snapshot.reconfigureOrReloadItems(MethodSettingsItemIdentifier.allShadowsocksItems)
            }
        case .socks5:
            snapshot.appendItems(
                MethodSettingsItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate),
                toSection: .proxyConfiguration
            )

            // Reconfigure the proxy configuration cell if validation error changed.
            if previousValidationError != newValidationError {
                snapshot.reconfigureOrReloadItems(
                    MethodSettingsItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate)
                )
            }
        }

        snapshot.appendSections([.validationError])
        snapshot.appendItems([.validationError], toSection: .validationError)

        snapshot.appendSections([.testingStatus])
        snapshot.appendSections([.cancelTest])

        // Add test status below the test button.
        if newValue.testingStatus != .initial {
            snapshot.appendItems([.testingStatus], toSection: .testingStatus)

            // Show cancel test button below test status.
            if newValue.testingStatus == .inProgress {
                snapshot.appendItems([.cancelTest], toSection: .cancelTest)
            }
        }

        if let previousValue, previousValue.testingStatus != newValue.testingStatus {
            snapshot.reconfigureOrReloadItems(snapshot.itemIdentifiers)
        }

        dataSource?.apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    func updateDataSourceWithContentValidationErrors(viewModel: AccessMethodViewModel) {
        guard var snapshot = dataSource?.snapshot() else {
            return
        }

        let itemsToReload: [MethodSettingsItemIdentifier] = switch viewModel.method {
        case .direct, .bridges:
            []
        case .shadowsocks:
            MethodSettingsItemIdentifier.allShadowsocksItems
        case .socks5:
            MethodSettingsItemIdentifier.allSocksItems(authenticate: viewModel.socks.authenticate)
        }

        snapshot.reconfigureOrReloadItems(itemsToReload + [.validationError])
        dataSource?.apply(snapshot, animatingDifferences: false)
    }
}
