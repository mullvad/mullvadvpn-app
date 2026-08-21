// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
                    action.capitalized,
                    displayFeature
                ),
                NSLocalizedString("Would you like to continue?", comment: ""),
            ].joinedParagraphs(lineBreaks: 1)
        }
    }
}
