//
//  NSAttributedString+Markdown.swift
//  MullvadVPN
//
//  Created by pronebird on 19/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSAttributedString {
    convenience init(markdownString: String, font: UIFont) {
        let attributedString = NSMutableAttributedString()
        let components = markdownString.components(separatedBy: "**")

        let fontDescriptor = font.fontDescriptor.withSymbolicTraits(.traitBold) ?? font
            .fontDescriptor
        let boldFont = UIFont(descriptor: fontDescriptor, size: font.pointSize)

        for (index, string) in components.enumerated() {
            var attributes = [NSAttributedString.Key: Any]()

            if index % 2 == 0 {
                attributes[.font] = font
            } else {
                attributes[.font] = boldFont
            }

            attributedString.append(NSAttributedString(string: string, attributes: attributes))
        }

        self.init(attributedString: attributedString)
    }
}
