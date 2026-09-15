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

@MainActor
final class SettingsViewModel: ObservableObject {
    struct Row {
        let route: SettingsNavigationRoute
        let title: String
        var subtitle: String?
        var detail = ""
        var isExternal = false
        var breadcrumb: Breadcrumb?
        let accessibilityIdentifier: AccessibilityIdentifier
    }

    struct Section {
        let rows: [Row]
        var footer: String?
    }

    @Published private(set) var tunnelSettings: LatestTunnelSettings
    @Published private(set) var isLoggedIn: Bool
    @Published private(set) var breadcrumbs: Set<Breadcrumb>
    @Published private(set) var showsMigratedSettings: Bool

    private let appPreferences: AppPreferencesDataSource
    private var tunnelObserver: TunnelObserver?
    private var breadcrumbsObserver: BreadcrumbsBlockObserver?

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

    var sections: [Section] {
        var vpnRows: [Row] = []
        if isLoggedIn {
            vpnRows += [
                row(
                    .daita,
                    title: NSLocalizedString("DAITA", comment: ""),
                    detail: onOff(tunnelSettings.daita.isEnabled),
                    accessibilityIdentifier: .daitaCell
                ),
                row(
                    .multihop,
                    title: NSLocalizedString("Multihop", comment: ""),
                    detail: tunnelSettings.tunnelMultihopState.description,
                    accessibilityIdentifier: .multihopCell
                ),
                row(
                    .vpnSettings,
                    title: NSLocalizedString("VPN settings", comment: ""),
                    accessibilityIdentifier: .vpnSettingsCell
                ),
            ]
        }
        vpnRows.append(
            row(
                .includeAllNetworks,
                title: NSLocalizedString("Force all apps", comment: ""),
                detail: onOff(tunnelSettings.includeAllNetworks.includeAllNetworksState.isEnabled),
                accessibilityIdentifier: .includeAllNetworksCell
            )
        )

        var generalRows = [
            row(
                .notificationSettings,
                title: NSLocalizedString("Notifications", comment: ""),
                accessibilityIdentifier: .notificationSettingsCell
            ),
            row(
                .changelog,
                title: NSLocalizedString("What’s new", comment: ""),
                subtitle: Bundle.main.productVersion,
                accessibilityIdentifier: .versionCell
            ),
        ]
        if isLoggedIn, showsMigratedSettings {
            generalRows.append(
                row(
                    .migratedSettings,
                    title: NSLocalizedString("Migrated settings", comment: ""),
                    accessibilityIdentifier: .migratedSettingsCell
                )
            )
        }

        return [
            Section(
                rows: vpnRows,
                footer: NSLocalizedString(
                    "Forces all apps on the device to use the VPN tunnel, preventing data leaks",
                    comment: ""
                )
            ),
            Section(rows: [
                row(
                    .apiAccess,
                    title: NSLocalizedString("API access", comment: ""),
                    accessibilityIdentifier: .apiAccessCell
                )
            ]),
            Section(rows: generalRows),
            Section(
                rows: [
                    row(
                        .problemReport,
                        title: NSLocalizedString("Report a problem", comment: ""),
                        accessibilityIdentifier: .problemReportCell
                    ),
                    row(
                        .faq,
                        title: NSLocalizedString("FAQs & Guides", comment: ""),
                        isExternal: true,
                        accessibilityIdentifier: .faqCell
                    ),
                    row(
                        .language,
                        title: NSLocalizedString("Language", comment: ""),
                        detail: ApplicationLanguage.currentLanguage.displayName,
                        isExternal: true,
                        accessibilityIdentifier: .languageCell
                    ),
                ],
                footer: NSLocalizedString(
                    "Changing language will disconnect you from the VPN and restart the app",
                    comment: ""
                )
            ),
        ]
    }

    private func row(
        _ route: SettingsNavigationRoute,
        title: String,
        subtitle: String? = nil,
        detail: String = "",
        isExternal: Bool = false,
        accessibilityIdentifier: AccessibilityIdentifier
    ) -> Row {
        Row(
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
