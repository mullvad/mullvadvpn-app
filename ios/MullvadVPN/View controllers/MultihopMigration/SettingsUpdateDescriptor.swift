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

struct SettingsUpdateDescriptor {
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

    var description: [TextItem] {
        switch change.path {
        case .none:
            []
        case .updatedMultiHop:
            [
                TextItem(
                    text: NSLocalizedString(
                        "Multihop is now split into three modes: When needed, Always, and Never."
                            + "This gives you more flexibility with your connection preferences.", comment: ""),
                    style: .secondary),
                TextItem(
                    text: String(
                        format: NSLocalizedString("Your multihop setting was migrated from “%@” to “%@”.", comment: ""),
                        "\(change.before!)", "\(change.after!)"), style: .primary),
                TextItem(
                    text: [
                        (change.after as? MultihopStateV2)?.description ?? "",
                        (change.after as? MultihopStateV2)?.comment ?? "",
                    ].joinedParagraphs(lineBreaks: 1),
                    symbols: [Image.mullvadIconMultihopWhenNeeded],
                    style: .secondary),
            ]
        case .uniqueFilter:
            [
                TextItem(
                    text: NSLocalizedString(
                        "Filters can now be set separately for entry and exit locations.", comment: ""),
                    style: .secondary),
                TextItem(
                    text: NSLocalizedString(
                        "Your current filters were applied to both entry and exit locations.", comment: ""),
                    style: .primary),
            ]
        case .directOnlyRemoved:
            [
                TextItem(
                    text: NSLocalizedString(
                        """
                        The DAITA sub-setting “Direct only” has been removed and simplified to avoid blocking connections. 
                        Instead, with DAITA enabled, you make this option with the multihop setting.
                        """, comment: ""),
                    style: .secondary),
                TextItem(
                    text: NSLocalizedString(
                        """
                        When multihop is set to “When needed” the app might use an additional server to make sure you connect to your selected location using DAITA.
                        """, comment: ""),
                    style: .secondary),
                TextItem(
                    text: NSLocalizedString(
                        "When multihop is set to “Always” or “Never” you must manually select a DAITA server.",
                        comment: ""),
                    style: .secondary),
            ]
        case .automatic:
            [
                TextItem(
                    text: NSLocalizedString(
                        """
                        The multihop mode “Always” now features an “Automatic” location for the entry server selection.
                        """, comment: ""),
                    style: .secondary),
                TextItem(
                    text: NSLocalizedString(
                        """
                        When selected, the app automatically picks a random server, prioritizing those closer to the exit location for better performance.
                        """, comment: ""),
                    style: .secondary),
            ]
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
}

extension MultihopStateV2 {
    var comment: String {
        switch self {
        case .never:
            NSLocalizedString(
                """
                \(description)
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
                To ensure your current settings work with your selected location, and to avoid blocking your
                connection, the app might automatically multihop via a different entry server.
                This will be indicated by the %@ symbol.
                """, comment: "")

        }
    }
}
