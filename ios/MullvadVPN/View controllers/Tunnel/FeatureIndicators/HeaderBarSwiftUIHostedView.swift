//
//  HeaderBarSwiftUIHostedView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-12-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

struct HeaderBarSwiftUIHostedView: UIViewRepresentable {
    typealias UIViewType = HeaderBarView

    func makeUIView(context: Context) -> HeaderBarView {
        let headerBarView = HeaderBarView(frame: CGRect(x: 0, y: 0, width: 100, height: 100))
        headerBarView.translatesAutoresizingMaskIntoConstraints = false
        headerBarView.insetsLayoutMarginsFromSafeArea = false

        var headerBarPresentation = HeaderBarPresentation.default
        headerBarView.backgroundColor = headerBarPresentation.style.backgroundColor()
        headerBarView.showsDivider = headerBarPresentation.showsDivider

        return headerBarView
    }

    func updateUIView(_ uiView: HeaderBarView, context: Context) {
        print("update")
    }
}
