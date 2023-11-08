//
//  AttributedMarkdownProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type of markdown element.
enum MarkdownElement {
    /// Bold text.
    case bold
    /// URL link.
    case link
}

/// Callback type used to override the attributed string attributes during parsing.
typealias MarkdownEffectCallback = (MarkdownElement, String) -> [NSAttributedString.Key: Any]

/// Type implementing conversion from markdown to attributed string.
protocol AttributedMarkdown {
    /// Convert the type to attributed string.
    ///
    /// - Parameters:
    ///   - options: markdown styling options.
    ///   - applyEffect: the callback used to override the string attributes during parsing.
    /// - Returns: the attributed string.
    func attributedString(options: MarkdownStylingOptions, applyEffect: MarkdownEffectCallback?) -> NSAttributedString
}

extension NSAttributedString.Key {
    /// The attributed string key used in place of `.link` whos text color is not customizable in UILabels.
    /// The value associated with this key can be a `String` or an `URL`.
    static let hyperlink = NSAttributedString.Key("HyperLink")
}

extension AttributedMarkdown {
    /// Convert the type to attributed string.
    ///
    /// - Parameter options: markdown styling options.
    /// - Returns: the attributed string.
    func attributedString(options: MarkdownStylingOptions) -> NSAttributedString {
        attributedString(options: options, applyEffect: nil)
    }
}
