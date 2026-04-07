//
//  String+Assets.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    var localizedQuoted: String {
        let openQuote = Locale.current.quotationBeginDelimiter ?? "\u{201C}"
        let closeQuote = Locale.current.quotationEndDelimiter ?? "\u{201D}"
        return "\(openQuote)\(self)\(closeQuote)"
    }

    enum Alerts {
        static func disconnectWarning(action: String, feature: String, quoteFeature: Bool = true) -> String {
            let localizedFeature = NSLocalizedString(feature, comment: "")
            let displayFeature = quoteFeature ? localizedFeature.localizedQuoted : localizedFeature

            return [
                String(
                    format:
                        NSLocalizedString(
                            "%@ %@ requires restarting the VPN connection, which will disconnect "
                                + "you and briefly expose your traffic. To prevent this, manually enable "
                                + "Airplane Mode and turn off Wi-Fi before continuing.", comment: ""),
                    NSLocalizedString(action, comment: "").capitalized,
                    displayFeature
                ),
                NSLocalizedString("Would you like to continue?", comment: ""),
            ].joinedParagraphs(lineBreaks: 1)
        }
    }
}
