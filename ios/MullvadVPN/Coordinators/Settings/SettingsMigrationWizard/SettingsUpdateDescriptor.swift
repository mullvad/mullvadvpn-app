//
//  SettingsUpdateDescriptor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import SwiftUI

struct SettingsUpdateDescriptor: SettingsMigrationPresentable {
    let change: Change

    var title: String {
        switch change.path {
        case .none:
            ""
        case .updatedMultiHop:
            NSLocalizedString("New multihop modes", comment: "")
        case .uniqueFilter:
            NSLocalizedString("Separate filters", comment: "")
        case .directOnlyRemoved:
            NSLocalizedString("“Direct only” removed", comment: "")
        case .automatic:
            NSLocalizedString("Multihop entry set to “Automatic”", comment: "")
        }
    }

    var banner: Image? {
        switch change.path {
        case .automatic:
            .mullvadAutomaticMultihopBanner
        case .uniqueFilter:
            .mullvadUniqueFilterBanner
        default:
            nil
        }
    }

    var description: [MullvadStateView.TextItem] {
        switch change.path {
        case .none:
            []
        case .updatedMultiHop:
            [
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        Multihop is now split into three modes: When needed, Always, and Never. \
                        This gives you more flexibility with your connection preferences.
                        """, comment: ""),
                    style: .secondary()),
                MullvadStateView.TextItem(
                    text: String(
                        format: NSLocalizedString("Your multihop setting was migrated from “%@” to “%@”.", comment: ""),
                        "\(change.before!)", "\(change.after!)"), style: .primary(.none)),
                MullvadStateView.TextItem(
                    text: (change.after as? MultihopStateV2)?.description ?? "",
                    style: .primary(
                        .bold,
                        padding: EdgeInsets(
                            top: 0,
                            leading: 0,
                            bottom: 0,
                            trailing: 0))),
                MullvadStateView.TextItem(
                    text: (change.after as? MultihopStateV2)?.comment ?? "",
                    symbols: (change.after as? MultihopStateV2)?.symbols ?? [],
                    style: .secondary()),
            ]
        case .uniqueFilter:
            [
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        "Filters can now be set separately for entry and exit locations.", comment: ""),
                    style: .secondary(.none)),
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        "Your current filters were applied to both entry and exit locations.", comment: ""),
                    style: .primary(.none)),
            ]
        case .directOnlyRemoved:
            [
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        The DAITA sub-setting “Direct only” has been removed and simplified to avoid blocking connections. \
                        Instead, with DAITA enabled, you make this option with the multihop setting.
                        """, comment: ""),
                    style: .secondary()),
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        When multihop is set to “When needed” the app might use an additional server to make sure you connect \
                        to your selected location using DAITA.
                        """, comment: ""),
                    style: .secondary()),
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        "When multihop is set to “Always” or “Never” you must manually select a DAITA server.",
                        comment: ""),
                    style: .secondary()),
            ]
        case .automatic:
            [
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        The multihop mode “Always” now features an “Automatic” location for the entry server selection.
                        """, comment: ""),
                    style: .secondary()),
                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        When selected, the app automatically picks a random server, prioritizing \
                        those closer to the exit location for better performance.
                        """, comment: ""),
                    style: .secondary()),
            ]
        }
    }
}

private extension MultihopStateV2 {
    var comment: String {
        switch self {
        case .never:
            NSLocalizedString(
                """
                Multihop is disabled. Your selected location must support all active settings in order to establish a connection.
                """, comment: "")

        case .always:
            NSLocalizedString(
                """
                Multihop is enabled. Your connection is routed through an entry server before exiting through the selected location.
                """, comment: "")
        case .whenNeeded:
            NSLocalizedString(
                """
                Not all our locations/servers support every feature in the app. If your selected location/server doesn’t support the features you’ve enabled, the app will automatically multihop via a compatible server.

                This ensures your connection does not get blocked due to incompatible settings.

                This will be indicated by the %@ symbol.

                """, comment: "")

        }
    }

    var symbols: [Image] {
        switch self {
        case .whenNeeded:
            [UIImage.Multihop.whenNeeded.scaledIcon(fromBaseSize: 18, to: .body, offset: .init(x: 0, y: 2))]
        default:
            []
        }
    }
}
