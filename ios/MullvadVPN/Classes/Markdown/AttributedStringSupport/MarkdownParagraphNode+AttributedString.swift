//
//  MarkdownParagraphNode+AttributedString.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension MarkdownParagraphNode: AttributedMarkdown {
    func attributedString(options: MarkdownStylingOptions, applyEffect: MarkdownEffectCallback?) -> NSAttributedString {
        let mutableAttributedString = children.compactMap { $0 as? AttributedMarkdown }
            .reduce(into: NSMutableAttributedString()) { partialResult, node in
                let attributedString = node.attributedString(options: options, applyEffect: applyEffect)
                partialResult.append(attributedString)
            }

        let range = NSRange(location: 0, length: mutableAttributedString.length)
        mutableAttributedString.addAttribute(.paragraphStyle, value: options.paragraphStyle, range: range)

        return mutableAttributedString
    }
}
