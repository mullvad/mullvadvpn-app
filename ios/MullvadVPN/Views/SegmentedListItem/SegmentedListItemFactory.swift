//
//  SegmentedListItemFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import SwiftUI

@MainActor
struct SegmentedListItemFactory {
    enum Leading {
        case customListLocation(node: LocationNode, level: Int = 0, isSelected: Binding<Bool>)
        case location(node: LocationNode, context: MultihopContext, level: Int = 0)
        case recentLocation(node: LocationNode, context: MultihopContext)
        case relayFilter(item: RelayFilterItem, isSelected: Binding<Bool>)
        case generic(title: String, subtitle: String? = nil, level: Int = 0, isSelected: Bool = false)
    }

    enum Trailing {
        case close(onSelect: () -> Void)
        case custom(items: [TrailingItem])
        case drillDown(title: String, breadcrumb: Breadcrumb? = nil)
        case external(title: String)
        case input(title: String, placeholder: String, text: Binding<String>)
        case text(title: String, breadcrumb: Breadcrumb? = nil)
        case toggle(isOn: Binding<Bool>, isDisabled: Bool)
    }

    enum Segment {
        case expand(isExpanded: Bool, onSelect: (() -> Void)?)
        case info(onSelect: (() -> Void)?)
    }

    enum StatusIndicator {
        enum DotStyle {
            case issue
            case offline
            case online
        }

        case checkbox(isOn: Binding<Bool>)
        case dot(DotStyle)
        case tick
    }

    enum TrailingItem {
        enum Icon {
            case chevron
            case close
            case external
            case info
        }

        enum Sizing {
            case button
            case custom(width: CGFloat, height: CGFloat)
        }

        case breadcrumb(_ breadcrumb: Breadcrumb)
        case button(icon: Icon, onSelect: () -> Void, sizing: Sizing = .button)
        case icon(_ icon: Icon, sizing: Sizing? = nil)
        case input(placeholder: String, text: Binding<String>)
        case padding(width: CGFloat = 16)
        case string(_ string: String)
        case toggle(isOn: Binding<Bool>, isDisabled: Bool)
    }

    @ViewBuilder func leading(for leading: Leading) -> some View {
        switch leading {
        case .customListLocation(let node, let level, let isSelected):
            CustomListLocationItemView(node: node, level: level, isSelected: isSelected)
        case .location(let node, let context, let level):
            LocationItemView(node: node, multihopContext: context, level: level)
        case .recentLocation(let node, let context):
            RecentItemView(node: node, multihopContext: context)
        case .relayFilter(let item, let isSelected):
            let isProvider = [.allProviders, .provider].contains(item.type)
            ListItem(
                title: item.name,
                level: 1,
                selected: isProvider ? false : isSelected.wrappedValue,
                statusIndicator: {
                    if isProvider {
                        statusIndicator(for: .checkbox(isOn: isSelected))
                    } else {
                        if isSelected.wrappedValue {
                            statusIndicator(for: .tick)
                        }
                    }
                }
            )
        case .generic(let title, let subtitle, let level, let isSelected):
            ListItem(
                title: title,
                subtitle: subtitle,
                level: level,
                selected: isSelected,
                statusIndicator: {
                    if isSelected {
                        statusIndicator(for: .tick)
                    }
                }
            )
        }
    }

    @ViewBuilder func trailing(for trailing: Trailing) -> some View {
        switch trailing {
        case .close(let onSelect):
            trailingItemViews(
                for: [
                    .button(icon: .close, onSelect: onSelect)
                ]
            )
        case .custom(let items):
            trailingItemViews(for: items)
        case .drillDown(let title, let breadcrumb):
            trailingItemViews(
                for: [
                    breadcrumb.flatMap { .breadcrumb($0) },
                    .string(title),
                    .icon(.chevron, sizing: .button),
                ].compactMap { $0 }
            )
        case .external(let title):
            trailingItemViews(
                for: [
                    .string(title),
                    .icon(.external, sizing: .button),
                ]
            )
        case .input(let title, let placeholder, let text):
            trailingItemViews(
                for: [
                    .string(title),
                    .padding(width: 8),
                    .input(placeholder: placeholder, text: text),
                    .padding(),
                ]
            )
        case .text(let title, let breadcrumb):
            trailingItemViews(
                for: [
                    breadcrumb.flatMap { .breadcrumb($0) },
                    .string(title),
                    .padding(),
                ].compactMap { $0 }
            )
        case .toggle(let isOn, let isDisabled):
            trailingItemViews(
                for: [
                    .toggle(isOn: isOn, isDisabled: isDisabled),
                    .padding(),
                ]
            )
        }
    }

    @ViewBuilder func segment(for segment: Segment) -> some View {
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

    @ViewBuilder func statusIndicator(for indicator: StatusIndicator) -> some View {
        switch indicator {
        case .checkbox(let isOn):
            Toggle("", isOn: isOn)
                .toggleStyle(CheckboxToggleStyle(accessibilityId: .checkbox))
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

    func image(for icon: TrailingItem.Icon) -> Image {
        switch icon {
        case .chevron:
            .mullvadIconChevron
        case .close:
            .mullvadIconCross
        case .external:
            .mullvadIconExternal
        case .info:
            .mullvadIconInfo
        }
    }

    func size(for size: TrailingItem.Sizing) -> CGSize {
        switch size {
        case .button:
            CGSize(
                width: UIMetrics.TableView.rowHeight,
                height: UIMetrics.TableView.rowHeight
            )
        case .custom(let width, let height):
            CGSize(width: width, height: height)
        }
    }

    private func trailingItemViews(for items: [TrailingItem]) -> some View {
        HStack(alignment: .center, spacing: 0) {
            ForEach(Array(items.enumerated()), id: \.offset) { _, item in
                switch item {
                case .breadcrumb(let breadcrumb):
                    breadcrumb.image
                        .resizable()
                        .frame(width: 18, height: 18)
                case .button(let icon, let onSelect, let sizing):
                    let size = self.size(for: sizing)
                    Button {
                        onSelect()
                    } label: {
                        image(for: icon)
                    }
                    .frame(
                        width: size.width,
                        height: size.height
                    )
                case .icon(let icon, let sizing):
                    let image = image(for: icon)
                        .foregroundStyle(Color.white)

                    if let sizing {
                        let size = self.size(for: sizing)
                        image
                            .frame(
                                width: size.width,
                                height: size.height
                            )
                    } else {
                        image
                    }
                case .input(let placeholder, let text):
                    MullvadPrimaryTextField(label: nil, placeholder: LocalizedStringKey(placeholder), text: text)
                        .multilineTextAlignment(.leading)
                case .padding(let width):
                    Spacer()
                        .frame(width: width)
                case .string(let string):
                    Text(string)
                        .lineLimit(1)
                        .fixedSize(horizontal: true, vertical: true)
                        .font(.mullvadTiny)
                        .foregroundStyle(Color.mullvadTextSecondary)
                case .toggle(let isOn, let isDisabled):
                    Toggle("", isOn: isOn)
                        .toggleStyle(
                            CustomToggleStyle(
                                disabled: isDisabled,
                                accessibilityId: nil,
                                infoButtonAction: nil
                            )
                        )
                        .disabled(isDisabled)
                }
            }
        }
    }
}
