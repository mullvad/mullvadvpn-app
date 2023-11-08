//
//  MarkdownBoldNode+AttributedString.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension MarkdownBoldNode: AttributedMarkdown {
    func attributedString(options: MarkdownStylingOptions, applyEffect: MarkdownEffectCallback?) -> NSAttributedString {
        let string = text?.withUnicodeLineSeparators() ?? ""
        var attributes: [NSAttributedString.Key: Any] = [.font: options.boldFont]

        if let textColor = options.textColor {
            attributes[.foregroundColor] = textColor
        }

        attributes.merge(applyEffect?(.bold, string) ?? [:], uniquingKeysWith: { $1 })

        return NSAttributedString(string: string, attributes: attributes)
    }
}
