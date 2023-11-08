//
//  MarkdownLinkNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The URL link node.
class MarkdownLinkNode: MarkdownNode {
    /// The link title.
    let title: String

    /// The URL string.
    let url: String

    /// Initialzes the link node with title and URL.
    /// - Parameters:
    ///   - title: the link title.
    ///   - url: the link URL.
    init(title: String, url: String) {
        self.title = title
        self.url = url
        super.init(type: .link)
    }

    override var debugDescription: String {
        "Link: \(title) (\(url))"
    }

    override func isEqualTo(_ other: MarkdownNode) -> Bool {
        guard let other = other as? MarkdownLinkNode else { return false }

        return title == other.title && url == other.url
    }
}
