//
//  MarkdownParagraphNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The paragraph node.
class MarkdownParagraphNode: MarkdownNode {
    /// Initializes the paragraph node.
    init(children: [MarkdownNode] = []) {
        super.init(type: .paragraph, children: children)
    }

    override var debugDescription: String {
        return "Paragraph"
    }
}
