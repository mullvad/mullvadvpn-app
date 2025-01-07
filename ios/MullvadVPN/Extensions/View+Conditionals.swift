//
//  View+Conditionals.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension View {
    @ViewBuilder func `if`<Content: View>(
        _ conditional: Bool,
        @ViewBuilder _ content: (Self) -> Content
    ) -> some View {
        if conditional {
            content(self)
        } else {
            self
        }
    }

    @ViewBuilder func ifLet<Content: View, T>(
        _ conditional: T?,
        @ViewBuilder _ content: (Self, _ value: T) -> Content
    ) -> some View {
        if let value = conditional {
            content(self, value)
        } else {
            self
        }
    }

    @ViewBuilder func showIf(_ conditional: Bool) -> some View {
        if conditional {
            self
        } else {
            EmptyView()
        }
    }
}
