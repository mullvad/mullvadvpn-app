//
//  MarkdownNodeType.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The type of node used within the markdown tree.
enum MarkdownNodeType {
    /// The unstyled text fragment.
    case text

    /// The bold text node.
    /// Syntax: `**Proceed carefully in unknown waters!**`
    case bold

    /// The URL link node.
    /// Syntax: `[Mullvad VPN](https://mullvad.net)`
    case link

    /// The paragraph node.
    /// Typically groups of elements separated by two newline characters form a paragraph.
    case paragraph

    /// The fragment of a markdown document.
    case document
}
