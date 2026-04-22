//
//  Untitled.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct PaginationView: View {
    let pages: [AnyView]

    init(pages: [any View]) {
        self.pages = pages.map { AnyView($0) }
    }

    var body: some View {
        TabView {
            ForEach(pages.indices, id: \.self) { index in
                pages[index]
            }
        }
        .tabViewStyle(.page)
    }
}

#Preview {
    PaginationView(pages: [Color.red, Text("Test")])
}
