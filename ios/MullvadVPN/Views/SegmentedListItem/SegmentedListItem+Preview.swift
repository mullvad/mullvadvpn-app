//
//  SegmentedListItem+Preview.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

#Preview("Settings") {
    @Previewable @State var inputText: String = ""
    @Previewable @State var toggleState: Bool = false

    let itemFactory = SegmentedListItemFactory()

    VStack(spacing: 0) {
        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - custom"))
            },
            trailing: {
                itemFactory.trailing(
                    for: .custom(items: [
                        .breadcrumb(.warning(.root)),
                        .string("Custom text"),
                        .button(icon: .info, onSelect: { print("onSelect") }),
                        .button(icon: .close, onSelect: { print("onClose") }),
                    ])
                )
            },
            footer: "Short description instead of an info icon"
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - close"))
            },
            trailing: {
                itemFactory.trailing(for: .close(onSelect: { print("onSelect") }))
            }
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - drilldown"))
            },
            trailing: {
                itemFactory.trailing(for: .drillDown(title: "Navigation title"))
            }
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - external"))
            },
            trailing: {
                itemFactory.trailing(for: .external(title: "Link title"))
            }
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(
                    for: .setting(
                        title: "Setting - text",
                        subtitle: "Subtitle",
                        isSelected: true
                    )
                )
            },
            trailing: {
                itemFactory.trailing(for: .text(title: "Some text", breadcrumb: .info(.vpnSettings)))
            }
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(
                    for: .setting(title: "Setting - input", subtitle: inputText.isEmpty ? "Input value" : inputText)
                )
            },
            trailing: {
                itemFactory.trailing(
                    for: .input(title: "Input title", placeholder: "Value", text: $inputText)
                )
            }
        )

        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - toggle", subtitle: toggleState ? "On" : "Off"))
            },
            trailing: {
                itemFactory.trailing(
                    for: .toggle(isOn: $toggleState, isDisabled: false)
                )
            }
        )

        Spacer()
    }
    .background(Color.mullvadBackground)
}

#Preview("Selection") {
    @Previewable @State var selection: MultihopState = .whenNeeded

    let itemFactory = SegmentedListItemFactory()
    let modes = MultihopState.allCases.map { $0 }

    VStack(spacing: 0) {
        SegmentedListItem(
            isLastInList: false,
            leading: {
                itemFactory.leading(
                    for: .setting(title: "Multihop mode", isSelected: false)
                )
            },
            groupedContent: {
                let level = 1
                ForEach(Array(modes.enumerated()), id: \.offset) { index, mode in
                    SegmentedListItem(
                        level: level,
                        isLastInList: index == modes.count - 1,
                        leading: {
                            itemFactory.leading(
                                for: .setting(
                                    title: mode.description,
                                    level: level,
                                    isSelected: selection == mode
                                )
                            )
                        },
                        segment: {
                            if mode == .whenNeeded {
                                itemFactory.segment(for: .info(onSelect: { print("onSelect") }))
                            }
                        },
                        onSelect: {
                            selection = mode
                        }
                    )
                }
            }
        )

        Spacer()
    }
    .background(Color.mullvadBackground)
}

#Preview("Location") {
    @Previewable @State var isExpandedLevel0: Bool = false
    @Previewable @State var isExpandedLevel1: Bool = false

    let itemFactory = SegmentedListItemFactory()
    let servers = [
        LocationNode(name: "se-sto-wg-001", code: "se-sto-wg-001"),
        LocationNode(name: "se-sto-wg-002", code: "se-sto-wg-002"),
    ]
    var city = LocationNode(name: "Stockholm", code: "se-sto", children: servers)
    var country = LocationNode(name: "Sweden", code: "se", children: [city])

    VStack(spacing: 0) {
        SegmentedListItem(
            isLastInList: !isExpandedLevel0,
            leading: {
                itemFactory.leading(
                    for: .location(
                        node: country,
                        context: .exit
                    )
                )
            },
            segment: {
                itemFactory.segment(
                    for: .expand(
                        isExpanded: isExpandedLevel0,
                        onSelect: {
                            isExpandedLevel0.toggle()
                        }
                    )
                )
            },
            groupedContent: {
                let level = 1
                if isExpandedLevel0 {
                    SegmentedListItem(
                        level: level,
                        isLastInList: !isExpandedLevel1,
                        leading: {
                            itemFactory.leading(
                                for: .location(
                                    node: city,
                                    context: .exit,
                                    level: level
                                )
                            )
                        },
                        segment: {
                            itemFactory.segment(
                                for: .expand(
                                    isExpanded: isExpandedLevel1,
                                    onSelect: {
                                        isExpandedLevel1.toggle()
                                    }
                                )
                            )
                        },
                        groupedContent: {
                            let level = 2
                            if isExpandedLevel1 {
                                ForEach(Array(servers.enumerated()), id: \.offset) { index, server in
                                    SegmentedListItem(
                                        level: level,
                                        isLastInList: index == servers.count - 1,
                                        leading: {
                                            itemFactory.leading(
                                                for: .location(
                                                    node: server,
                                                    context: .exit,
                                                    level: level
                                                )
                                            )
                                        }
                                    )
                                }
                            }
                        }
                    )
                }
            }
        )

        Spacer()
    }
    .background(Color.mullvadBackground)

}

#Preview("Custom list location") {
    @Previewable @State var checkboxState: Bool = false
    @Previewable @State var isExpandedLevel0: Bool = false
    @Previewable @State var isExpandedLevel1: Bool = false

    let itemFactory = SegmentedListItemFactory()
    let servers = [
        LocationNode(name: "se-sto-wg-001", code: "se-sto-wg-001"),
        LocationNode(name: "se-sto-wg-002", code: "se-sto-wg-002"),
    ]
    var city = LocationNode(name: "Stockholm", code: "se-sto", children: servers)
    var country = LocationNode(name: "Sweden", code: "se", children: [city])

    VStack(spacing: 0) {
        SegmentedListItem(
            isLastInList: !isExpandedLevel0,
            leading: {
                itemFactory.leading(
                    for: .customListLocation(
                        node: country,
                        isSelected: $checkboxState
                    )
                )
            },
            segment: {
                itemFactory.segment(
                    for: .expand(
                        isExpanded: isExpandedLevel0,
                        onSelect: {
                            isExpandedLevel0.toggle()
                        }
                    )
                )
            },
            groupedContent: {
                let level = 1
                if isExpandedLevel0 {
                    SegmentedListItem(
                        level: level,
                        isLastInList: !isExpandedLevel1,
                        leading: {
                            itemFactory.leading(
                                for: .customListLocation(
                                    node: city,
                                    level: level,
                                    isSelected: $checkboxState
                                )
                            )
                        },
                        segment: {
                            itemFactory.segment(
                                for: .expand(
                                    isExpanded: isExpandedLevel1,
                                    onSelect: {
                                        isExpandedLevel1.toggle()
                                    }
                                )
                            )
                        },
                        groupedContent: {
                            let level = 2
                            if isExpandedLevel1 {
                                ForEach(Array(servers.enumerated()), id: \.offset) { index, server in
                                    SegmentedListItem(
                                        level: level,
                                        isLastInList: index == servers.count - 1,
                                        leading: {
                                            itemFactory.leading(
                                                for: .customListLocation(
                                                    node: server,
                                                    level: level,
                                                    isSelected: $checkboxState
                                                )
                                            )
                                        }
                                    )
                                }
                            }
                        }
                    )
                }
            }
        )

        Spacer()
    }
    .background(Color.mullvadBackground)
}
