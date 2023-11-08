//
//  MarkdownStylingOptions.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Struct describing the visual style that should be used when converting from markdown to attributed string.
struct MarkdownStylingOptions {
    /// Primary font for text.
    var font: UIFont

    /// Text color.
    var textColor: UIColor?

    /// The color of the link.
    /// UIKit controls may ignore it when used with standard link attributes.
    var linkColor: UIColor?

    /// The attribute that holds the URL.
    var linkAttribute: MarkdownLinkAttribute = .standard

    /// Paragraph style
    var paragraphStyle: NSParagraphStyle = .default

    /// Bold font derived from primary font.
    var boldFont: UIFont {
        let fontDescriptor = font.fontDescriptor.withSymbolicTraits(.traitBold) ?? font.fontDescriptor
        return UIFont(descriptor: fontDescriptor, size: font.pointSize)
    }
}

/// The attribute that holds the URL.
enum MarkdownLinkAttribute {
    /// Standard `NSLinkAttribute` attribute.
    case standard

    /// Custom hyperlink attribute.
    case custom

    /// Returns the attribute key which should be used to store the link URL.
    var attributeKey: NSAttributedString.Key {
        switch self {
        case .standard:
            return .link
        case .custom:
            return .hyperlink
        }
    }
}
