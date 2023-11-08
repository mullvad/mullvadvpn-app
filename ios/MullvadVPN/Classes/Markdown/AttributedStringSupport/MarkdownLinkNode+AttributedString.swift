//
//  MarkdownLinkNode+AttributedString.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension MarkdownLinkNode: AttributedMarkdown {
    func attributedString(options: MarkdownStylingOptions, applyEffect: MarkdownEffectCallback?) -> NSAttributedString {
        var attributes: [NSAttributedString.Key: Any] = [.font: options.font, options.linkAttribute.attributeKey: url]

        if let linkColor = options.linkColor {
            attributes[.foregroundColor] = linkColor
        }

        attributes.merge(applyEffect?(.link, title) ?? [:], uniquingKeysWith: { $1 })

        return NSAttributedString(string: title, attributes: attributes)
    }
}
