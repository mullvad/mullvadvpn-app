//
//  AttributedString+Helpers.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-08-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension AttributedString {
    /// Construct an AttributedString from text assumed to be in Markdown. If Markdown parsing fails, constructs one treating the text as plain text.
    static func fromMarkdown(_ markdown: String) -> AttributedString {
        (try? AttributedString(markdown: markdown)) ?? AttributedString(markdown)
    }
}
