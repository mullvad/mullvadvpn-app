//
//  MarkdownBoldNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The bold text node.
class MarkdownBoldNode: MarkdownNode {
    /// The text content.
    private(set) var text: String?

    /// Indicates whether the closing tag was found for that node.
    private(set) var isClosed = false

    /// Initializes the bold text node.
    ///
    /// - Parameters:
    ///   - text: text content.
    ///   - isClosed: whether the closing tag was found.
    init(text: String? = nil, isClosed: Bool = true) {
        self.text = text
        self.isClosed = isClosed
        super.init(type: .bold)
    }

    override var debugDescription: String {
        "Bold: \(text ?? "(nil)")"
    }

    /// Append the string to node's text content.
    /// - Parameter string: the string to append to the node's text content.
    func appendText(_ string: String) {
        if text == nil {
            text = string
        } else {
            text?.append(string)
        }
    }

    /// Mark that the closing tag was found for that node.
    func markClosed() {
        isClosed = true
    }

    override func isEqualTo(_ other: MarkdownNode) -> Bool {
        guard let other = other as? MarkdownBoldNode else { return false }

        return text == other.text
    }
}
