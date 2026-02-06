//
//  Separator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RowSeparator: View {
    let color: Color
    let edgeInsets: EdgeInsets

    init(color: Color = Color(.secondaryColor), edgeInsets: EdgeInsets = .init()) {
        self.color = color
        self.edgeInsets = edgeInsets
    }

    var body: some View {
        color
            .frame(height: UIMetrics.TableView.separatorHeight)
            .padding(edgeInsets)
    }
}

#Preview {
    RowSeparator(color: Color(.primaryColor))
}
