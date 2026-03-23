//
//  ListItemFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ListItemFactory {
    enum Label {
        case location(node: LocationNode, context: MultihopContext, level: Int)
        case recent(node: LocationNode, level: Int)
        case setting(title: String, subtitle: String? = nil, level: Int = 0, selected: Bool = false)
    }

    enum Segment {
        case expand(isExpanded: Bool, onSelect: (() -> Void)?)
        case info(onSelect: (() -> Void)?)
    }

    enum StatusIndicator {
        enum DotStyle {
            case issue, offline, online
        }

        case dot(DotStyle)
        case tick
    }

    @MainActor @ViewBuilder func label(for label: Label) -> some View {
        switch label {
        case .location(let node, let context, let level):
            RelayItemView(node: node, multihopContext: context, level: level)
        case .recent(let node, let level):
            RecentItemView(node: node, level: level)
        case .setting(let title, let subtitle, let level, let selected):
            ListItem(
                title: title,
                subtitle: subtitle,
                level: level,
                selected: selected,
                statusIndicator: {
                    if selected {
                        statusIndicator(for: .tick)
                    }
                }
            )
        }
    }

    @MainActor @ViewBuilder func segment(for segment: Segment) -> some View {
        switch segment {
        case .expand(let isExpanded, let onSelect):
            Button {
                onSelect?()
            } label: {
                Image.mullvadIconChevron
                    .rotationEffect(.degrees(isExpanded ? -90 : 90))
                    .accessibilityLabel(
                        isExpanded ? Text("Collapse") : Text("Expand")
                    )
                    .accessibilityIdentifier(.expandButton)
            }
        case .info(let onSelect):
            Button {
                onSelect?()
            } label: {
                Image.mullvadIconInfo
                    .accessibilityLabel(Text("Information"))
                    .accessibilityIdentifier(.infoButton)
            }
        }
    }

    @MainActor @ViewBuilder func statusIndicator(for indicator: StatusIndicator) -> some View {
        switch indicator {
        case .dot(let style):
            switch style {
            case .issue:
                Image.mullvadIconStateIssue
            case .offline:
                Image.mullvadIconStateOffline
            case .online:
                Image.mullvadIconStateOnline
            }
        case .tick:
            Image.mullvadIconTick
                .foregroundStyle(Color.mullvadSuccessColor)
        }
    }
}
