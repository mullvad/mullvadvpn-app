//
//  NSAttributedString+Markdown.swift
//  MullvadVPN
//
//  Created by pronebird on 19/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSAttributedString {
    /// Initialize the attributed string from markdown.
    ///
    /// - Parameters:
    ///   - markdownString: the markdown string.
    ///   - options: markdown styling options.
    ///   - applyEffect: the callback used to override the string attributes during parsing.
    convenience init(
        markdownString: String,
        options: MarkdownStylingOptions,
        applyEffect: MarkdownEffectCallback? = nil
    ) {
        var parser = MarkdownParser(markdown: markdownString)
        let document = parser.parse()

        let attributedString = document.attributedString(options: options, applyEffect: applyEffect)

        self.init(attributedString: attributedString)
    }
}
