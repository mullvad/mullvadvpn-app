//
//  String+Assets.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    enum Alerts {
        static func disconnectWarning(action: String, feature: String) -> String {
            NSLocalizedString(
                [
                    String(
                        format:
                            "%@ %@ requires restarting the VPN connection, which will disconnect "
                            + "you and briefly expose your traffic. To prevent this, manually enable "
                            + "Airplane Mode and turn off Wi-Fi before continuing.",
                        action.capitalized,
                        feature
                    ),
                    String(
                        format: "Would you like to continue %@ %@?",
                        action,
                        feature
                    ),
                ].joinedParagraphs(lineBreaks: 1),
                comment: ""
            )
        }
    }
}
