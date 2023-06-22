//
//  NSAttributedString+Markdown.swift
//  MullvadVPN
//
//  Created by pronebird on 19/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

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
        let paragraphs = markdownString.replacingOccurrences(of: "\r\n", with: "\n").components(separatedBy: "\n\n")

        for (paragraphIndex, paragraph) in paragraphs.enumerated() {
            let attributedParagraph = NSMutableAttributedString()

            // Replace \n with \u2028 to prevent attributed string from picking up single line breaks as paragraphs.
            let components = paragraph.replacingOccurrences(of: "\n", with: "\u{2028}")
                .components(separatedBy: "**")

            if paragraphIndex > 0 {
                // Add single line break to add spacing between paragraphs.
                attributedParagraph.append(NSAttributedString(string: "\n"))
            }

            for (stringIndex, string) in components.enumerated() {
                var attributes: [NSAttributedString.Key: Any] = [:]

                if stringIndex % 2 == 0 {
                    attributes[.font] = options.font
                } else {
                    attributes[.font] = options.boldFont
                    attributes.merge(applyEffect?(.bold, string) ?? [:], uniquingKeysWith: { $1 })
                }

                attributedParagraph.append(NSAttributedString(string: string, attributes: attributes))
            }

            attributedString.append(attributedParagraph)
        }

        attributedString.addAttribute(
            .paragraphStyle,
            value: options.paragraphStyle,
            range: NSRange(location: 0, length: attributedString.length)
        )

        self.init(attributedString: attributedString)
    }
}
