// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI
import UIKit

// Various fixes (eg. viewModel.onHeightChange, viewModel.safeAreaTop) can be removed
// once HeaderBarView isn't wrapped in a UIHostingViewController.
struct HeaderBarView: View {
    private let barButtonTappableAreaSize = UIMetrics.Button.minimumTappableAreaSize
    @State private var logoTrailingPoint: CGFloat = 0
    @State private var buttonsLeadingPoint: CGFloat = 0
    @State private var showBrandName: Bool = true

    let viewModel: HeaderBarViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Group {
                    if showBrandName {
                        Image.mullvadLogo
                            .resizable()
                            .background(determineBrandNameOverlap(point: $logoTrailingPoint))
                    } else {
                        Image.mullvadLogoImage
                            .resizable()
                    }
                }
                .scaledToFit()
                .frame(height: UIMetrics.headerBarLogoSize)
                .padding(.top, 4)
                .accessibilityHidden(true)

                Spacer()

                Group {
                    Button {
                        viewModel.onAccountTap?()
                    } label: {
                        Image.mullvadIconAccount
                            .resizable()
                            .scaledToFit()
                            .frame(width: UIMetrics.Button.barButtonSize)
                    }
                    .frame(width: barButtonTappableAreaSize.width, height: barButtonTappableAreaSize.height)
                    .accessibilityIdentifier(.accountButton)
                    .accessibilityLabel(NSLocalizedString("Account", comment: ""))
                    .showIf(viewModel.showAccountButton)

                    Button {
                        viewModel.onSettingsTap?()
                    } label: {
                        Group {
                            if let breadcrumb = viewModel.breadcrumb {
                                breadcrumb.settingsImage
                                    .resizable()
                            } else {
                                Image.mullvadIconSettings
                                    .resizable()
                            }
                        }
                        .scaledToFit()
                        .frame(height: UIMetrics.Button.barButtonSize)
                    }
                    .frame(width: barButtonTappableAreaSize.width, height: barButtonTappableAreaSize.height)
                    .accessibilityIdentifier(.settingsButton)
                    .accessibilityLabel(NSLocalizedString("Settings", comment: ""))
                }
                .background(determineBrandNameOverlap(point: $buttonsLeadingPoint))
            }

            HStack(spacing: 16) {
                Text(viewModel.deviceName)
                    .accessibilityIdentifier(.headerDeviceNameLabel)
                Text(viewModel.timeLeft)
            }
            .showIf(viewModel.showDeviceInfo)
            .foregroundStyle(Color.mullvadTextSecondary)
            .font(.mullvadMiniSemiBold)
        }
        .padding(EdgeInsets(top: viewModel.safeAreaTop, leading: 16, bottom: 8, trailing: 10))
        .accessibilityElement(children: .contain)
        .background(Color(viewModel.backgroundColor))
        .ignoresSafeArea(edges: .top)
        .sizeOfView { viewModel.onHeightChange?($0.height) }
    }

    private func determineBrandNameOverlap(point: Binding<CGFloat>) -> GeometryReader<some View> {
        GeometryReader { proxy in
            let frame = proxy.frame(in: .global)

            Color.clear
                .onAppear {
                    point.wrappedValue = frame.origin.x + frame.width
                    showBrandName = logoTrailingPoint < buttonsLeadingPoint
                }
        }
    }
}

#Preview {
    HeaderBarView(
        viewModel: HeaderBarViewModel(
            rootConfiguration: RootConfiguration(
                deviceName: "Happy Cow",
                expiry: Date().addingTimeInterval(60),
                showsAccountButton: true
            ),
            headerBarPresentation: HeaderBarPresentation(
                style: .default
            ),
            breadcrumb: Breadcrumb.error(.apiAccess)
        )
    )
}
