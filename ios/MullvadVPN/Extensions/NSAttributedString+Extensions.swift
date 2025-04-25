//
//  NSAttributedString+Markdown.swift
//  MullvadVPN
//
//  Created by pronebird on 19/11/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSAttributedString {
    enum MarkdownElement {
        case bold
    }

    convenience init(
        markdownString: String,
        options: MarkdownStylingOptions,
        attributes: [NSAttributedString.Key: Any]? = nil,
        applyEffect: ((String) -> [NSAttributedString.Key: Any])? = nil
    ) {
        let attributedString = NSMutableAttributedString()
        let components = markdownString.components(separatedBy: "**")

        for (stringIndex, string) in components.enumerated() {
            var attributes = attributes ?? [:]

            if stringIndex % 2 != 0 {
                attributes[.font] = options.font
                attributes[.foregroundColor] = options.textColor
                attributes.merge(applyEffect?(string) ?? [:], uniquingKeysWith: { $1 })
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
