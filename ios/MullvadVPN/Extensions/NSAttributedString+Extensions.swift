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

extension NSAttributedString {
    enum MarkdownElement {
        case bold
    }

    convenience init(
        markdownString: String,
        options: MarkdownStylingOptions,
        applyEffect: ((MarkdownElement, String) -> [NSAttributedString.Key: Any])? = nil
    ) {
        let attributedString = NSMutableAttributedString()
        let components = markdownString.components(separatedBy: "**")

        for (stringIndex, string) in components.enumerated() {
            var attributes: [NSAttributedString.Key: Any] = [:]

            if stringIndex % 2 == 0 {
                attributes[.font] = options.font
            } else {
                attributes[.font] = options.font.withWeight(.bold)
                attributes.merge(applyEffect?(.bold, string) ?? [:], uniquingKeysWith: { $1 })
            }

            attributedString.append(NSAttributedString(string: string, attributes: attributes))
        }

        attributedString.addAttribute(
            .paragraphStyle,
            value: options.paragraphStyle,
            range: NSRange(location: 0, length: attributedString.length)
        )

        self.init(attributedString: attributedString)
    }
}

extension NSMutableAttributedString {
    func apply(paragraphStyle: NSParagraphStyle) {
        let attributeRange = NSRange(location: 0, length: length)
        addAttribute(.paragraphStyle, value: paragraphStyle, range: attributeRange)
    }
}
