//
//  MarkdownTextNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The untagged and unstyled text fragment.
class MarkdownTextNode: MarkdownNode {
    /// The text content.
    private(set) var text: String?

    /// Initializes the text node.
    /// - Parameter text: the text content.
    init(text: String? = nil) {
        self.text = text
        super.init(type: .text)
    }

    override var debugDescription: String {
        "Text: \(text ?? "(nil)")"
    }

    /// Append text content if set, otherwise assigns the text content to the given string.
    /// - Parameter string: the string to append to the node's text content.
    func appendText(_ string: String) {
        if text == nil {
            text = string
        } else {
            text?.append(string)
        }
    }

    override func isEqualTo(_ other: MarkdownNode) -> Bool {
        guard let other = other as? MarkdownTextNode else { return false }

        return text == other.text
    }
}
