//
//  MarkdownStylingOptions.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct MarkdownStylingOptions {
    var font: UIFont
    var textColor: UIColor?
    var linkColor: UIColor?
    var paragraphStyle: NSParagraphStyle = .default

    var boldFont: UIFont {
        let fontDescriptor = font.fontDescriptor.withSymbolicTraits(.traitBold) ?? font.fontDescriptor
        return UIFont(descriptor: fontDescriptor, size: font.pointSize)
    }
}
