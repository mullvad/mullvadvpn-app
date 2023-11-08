//
//  MarkdownDocumentNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The root node that represents a markdown document.
class MarkdownDocumentNode: MarkdownNode {
    /// Initializes the document node.
    init(children: [MarkdownNode] = []) {
        super.init(type: .document, children: children)
    }

    override var debugDescription: String {
        return "Document"
    }
}
