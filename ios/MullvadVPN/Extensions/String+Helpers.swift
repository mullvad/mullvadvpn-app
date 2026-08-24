// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

extension String {
    /// Returns the array of the longest possible subsequences of the given length.
    func split(every length: Int) -> [Substring] {
        guard length > 0 else { return [prefix(upTo: endIndex)] }

        let resultCount = Int((Float(count) / Float(length)).rounded(.up))

        return (0..<resultCount)
            .map { dropFirst($0 * length).prefix(length) }
    }

    func width(using font: UIFont) -> CGFloat {
        let fontAttributes = [NSAttributedString.Key.font: font]
        return self.size(withAttributes: fontAttributes).width
    }
}

extension Array where Element == String {
    func joinedParagraphs(lineBreaks: Int = 2) -> String {
        let separator = String(repeating: "\n", count: lineBreaks)
        return self.joined(separator: separator)
    }
}
