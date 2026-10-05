// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

struct SettingsRow {
    let route: SettingsNavigationRoute
    let title: String
    var subtitle: String?
    var detail: String?
    var isExternal = false
    var breadcrumb: Breadcrumb?
    let accessibilityIdentifier: AccessibilityIdentifier
}

struct SettingsSection {
    let kind: SettingsSectionKind
    let rows: [SettingsRow]
}

@MainActor
protocol SettingsViewModelProtocol: AnyObject, Observable {
    var tunnelSettings: LatestTunnelSettings { get }
    var isLoggedIn: Bool { get }
    var breadcrumbs: Set<Breadcrumb> { get }
    var showsMigratedSettings: Bool { get }

    func refresh()
}

@MainActor
@Observable
final class SettingsViewModel: SettingsViewModelProtocol {
    private(set) var tunnelSettings: LatestTunnelSettings
    private(set) var isLoggedIn: Bool
    private(set) var breadcrumbs: Set<Breadcrumb>
    private(set) var showsMigratedSettings: Bool

    private let appPreferences: AppPreferencesDataSource
    @ObservationIgnored private var tunnelObserver: TunnelObserver?
    @ObservationIgnored private var breadcrumbsObserver: BreadcrumbsBlockObserver?

    init(
        tunnelManager: TunnelManager,
        appPreferences: AppPreferencesDataSource,
        breadcrumbsProvider: BreadcrumbsProvider
    ) {
        self.appPreferences = appPreferences

        tunnelSettings = tunnelManager.settings
        isLoggedIn = tunnelManager.deviceState.isLoggedIn
        breadcrumbs = breadcrumbsProvider.breadcrumbs
        showsMigratedSettings = appPreferences.migratedSettingsState.shouldShowMigratedSettingsMenuItem

        let tunnelObserver = TunnelBlockObserver(
            didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.isLoggedIn = deviceState.isLoggedIn
            },
            didUpdateTunnelSettings: { [weak self] _, settings in
                self?.tunnelSettings = settings
            }
        )
        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        let breadcrumbsObserver = BreadcrumbsBlockObserver(didUpdateBreadcrumbsHandler: { [weak self] in
            self?.breadcrumbs = $0
        })
        breadcrumbsProvider.add(observer: breadcrumbsObserver)
        self.breadcrumbsObserver = breadcrumbsObserver
    }

    /// Re-reads state that is not observable, such as app preferences.
    func refresh() {
        showsMigratedSettings = appPreferences.migratedSettingsState.shouldShowMigratedSettingsMenuItem
    }
}

enum SettingsSectionKind: CaseIterable {
    case vpn, apiAccess, general, support

    var footer: String? {
        switch self {
        case .vpn:
            NSLocalizedString(
                "Forces all apps on the device to use the VPN tunnel, preventing data leaks",
                comment: ""
            )
        case .support:
            NSLocalizedString(
                "Changing language will disconnect you from the VPN and restart the app",
                comment: ""
            )
        case .apiAccess, .general:
            nil
        }
    }
}

extension SettingsViewModelProtocol {
    var sections: [SettingsSection] {
        SettingsSectionKind.allCases.compactMap { kind in
            let routes = routes(in: kind)
            guard !routes.isEmpty else { return nil }
            return SettingsSection(kind: kind, rows: routes.map(row(for:)))
        }
    }

    private func routes(in section: SettingsSectionKind) -> [SettingsNavigationRoute] {
        switch section {
        case .vpn:
            (isLoggedIn ? [.daita, .multihop, .vpnSettings] : []) + [.includeAllNetworks]
        case .apiAccess:
            [.apiAccess]
        case .general:
            [.notificationSettings, .changelog] + (isLoggedIn && showsMigratedSettings ? [.migratedSettings] : [])
        case .support:
            [.problemReport, .faq, .language]
        }
    }

    private func row(for route: SettingsNavigationRoute) -> SettingsRow {
        switch route {
        case .daita:
            row(
                route,
                title: NSLocalizedString("DAITA", comment: ""),
                detail: onOff(tunnelSettings.daita.isEnabled),
                accessibilityIdentifier: .daitaCell
            )
        case .multihop:
            row(
                route,
                title: NSLocalizedString("Multihop", comment: ""),
                detail: tunnelSettings.tunnelMultihopState.description,
                accessibilityIdentifier: .multihopCell
            )
        case .vpnSettings:
            row(
                route,
                title: NSLocalizedString("VPN settings", comment: ""),
                accessibilityIdentifier: .vpnSettingsCell
            )
        case .includeAllNetworks:
            row(
                route,
                title: NSLocalizedString("Force all apps", comment: ""),
                detail: onOff(tunnelSettings.includeAllNetworks.includeAllNetworksState.isEnabled),
                accessibilityIdentifier: .includeAllNetworksCell
            )
        case .apiAccess:
            row(
                route,
                title: NSLocalizedString("API access", comment: ""),
                accessibilityIdentifier: .apiAccessCell
            )
        case .notificationSettings:
            row(
                route,
                title: NSLocalizedString("Notifications", comment: ""),
                accessibilityIdentifier: .notificationSettingsCell
            )
        case .changelog:
            row(
                route,
                title: NSLocalizedString("What’s new", comment: ""),
                subtitle: Bundle.main.productVersion,
                accessibilityIdentifier: .versionCell
            )
        case .migratedSettings:
            row(
                route,
                title: NSLocalizedString("Migrated settings", comment: ""),
                accessibilityIdentifier: .migratedSettingsCell
            )
        case .problemReport:
            row(
                route,
                title: NSLocalizedString("Report a problem", comment: ""),
                accessibilityIdentifier: .problemReportCell
            )
        case .faq:
            row(
                route,
                title: NSLocalizedString("FAQs & Guides", comment: ""),
                isExternal: true,
                accessibilityIdentifier: .faqCell
            )
        case .language:
            row(
                route,
                title: NSLocalizedString("Language", comment: ""),
                detail: ApplicationLanguage.currentLanguage.displayName,
                isExternal: true,
                accessibilityIdentifier: .languageCell
            )
        case .root:
            preconditionFailure("The root route has no settings row")
        }
    }

    private func row(
        _ route: SettingsNavigationRoute,
        title: String,
        subtitle: String? = nil,
        detail: String? = nil,
        isExternal: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier
    ) -> SettingsRow {
        SettingsRow(
            route: route,
            title: title,
            subtitle: subtitle,
            detail: detail,
            isExternal: isExternal,
            breadcrumb: breadcrumbs.first { $0.navigationRoute == route },
            accessibilityIdentifier: accessibilityIdentifier
        )
    }

    private func onOff(_ isEnabled: Bool) -> String {
        isEnabled ? NSLocalizedString("On", comment: "") : NSLocalizedString("Off", comment: "")
    }
}

#if DEBUG
@Observable
final class MockRootSettingsViewModel: SettingsViewModelProtocol {
    var tunnelSettings = LatestTunnelSettings()
    var isLoggedIn = true
    var breadcrumbs: Set<Breadcrumb> = [.warning(.daita)]
    var showsMigratedSettings = true

    func refresh() {}
}
#endif
