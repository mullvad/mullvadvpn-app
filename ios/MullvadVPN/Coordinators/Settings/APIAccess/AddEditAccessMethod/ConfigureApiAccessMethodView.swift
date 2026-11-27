// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

enum AccessMethodField: Hashable {
    case name, server, port, username, password
}

struct ConfigureApiAccessMethodView: View {

    let isRootView: Bool
    @State var viewModel: any ConfigureApiAccessMethodViewModelProtocol
    @Environment(\.dismiss) private var dismiss
    @State private var savingAlert: MullvadAlert?
    @State private var discardAlert: MullvadAlert?
    @FocusState private var focusedInput: AccessMethodField?
    @State private var refreshID = UUID()

    let testViewAnchor = "testView"

    private let itemFactory = SegmentedListItemFactory()

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    promptView
                    nameField
                    typeField
                    serverDetailsSection
                    testView
                }
                .padding(.horizontal, 16)
            }
            .id(refreshID)
            .scrollBounceBehavior(.basedOnSize)
            .scrollDismissesKeyboard(.interactively)
            .onChange(of: viewModel.testState) {
                if viewModel.testState == .testing {
                    withAnimation {
                        proxy.scrollTo(
                            testViewAnchor,
                            anchor: .center
                        )
                    }
                } else if case .reachable = viewModel.testState {
                    viewModel.save()
                    dismiss()
                } else if case .unreachable = viewModel.testState {
                    savingAlert = methodSavingAlert
                }
            }
        }
        .background {
            Color.mullvadBackground
                .ignoresSafeArea()
        }
        .accessibilityIdentifier(.addAccessMethodScrollView)
        .navigationBarTitleDisplayMode(.inline)
        .mullvadAlert(item: $savingAlert)
        .mullvadAlert(item: $discardAlert)
        .onAppear {
            refreshID = UUID()
            LocalNetworkProbe()
                .triggerLocalNetworkPrivacyAlert()
        }
        .onDisappear {
            Task {
                focusedInput = nil
                await viewModel.cancelTesting()
            }
        }
        .onChange(of: viewModel.accessMethodViewModel.method) {
            viewModel.clearErrors(for: [
                .server,
                .port,
                .username,
                .password,
                .cipher,
            ])
        }
        .navigationBarBackButtonHidden()
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button {
                    dismissView()
                } label: {
                    if isRootView {
                        Image(.iconCross)
                    } else {
                        Image(.iconBack)
                    }
                }
                .buttonStyle(DisabledButtonStyle())
                .disabled(viewModel.isTesting)
                .accessibilityLabel(isRootView ? Text("Close") : Text("Back"))
                .accessibilityHint(Text("Dismisses the current screen"))
            }
            .hideLiquidGlassEffect()

            DoneToolbarItem(
                accessibilityIdentifier: .accessMethodAddButton,
                isDisabled: viewModel.isTesting || !viewModel.canSubmit,
                onDone: {
                    focusedInput = nil
                    Task {
                        await viewModel.startTesting()
                    }
                }
            )
        }
    }

    private var promptView: some View {
        HStack(alignment: .center) {
            Spacer()
            ResizableImageView(image: .mullvadIconInfo, dimension: .width(15))
            Text("The app will test the method before saving.")
                .font(.mullvadTinySemiBold)
                .frame(maxWidth: .infinity, alignment: .leading)
                .foregroundStyle(Color.mullvadTextSecondary)
            Spacer()
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(NSLocalizedString("Info: The app will test the method before saving.", comment: ""))
        .padding(.bottom, 16)
    }

    private var nameField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            appearance: viewModel.hasError(for: .name) == nil ? .normal : .error,
            accessibilityIdentifier: .accessMethodNameTextField,
            accessibilityLabel: NSLocalizedString("Name", comment: ""),
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Name", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .input(
                        title: "",
                        placeholder: NSLocalizedString("Required", comment: ""),
                        text: $viewModel.accessMethodViewModel.name)
                )
                .focused($focusedInput, equals: .name)
            },
            footer: {
                if let error = viewModel.hasError(for: .name) {
                    itemFactory
                        .footer(
                            for: .generic(
                                text: error.errorDescription
                            )
                        )
                }
            },
            onSelect: {
                focusedInput = .name
            }
        )
    }

    private var typeField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            accessibilityIdentifier: .accessMethodProtocolSelectionCell,
            accessibilityLabel: NSLocalizedString("Type", comment: ""),
            leadingAndTrailingDestination: {
                RetainedDestination {
                    AccessMethodTypeViewModel(
                        current: $viewModel.accessMethodViewModel.method,
                        list: AccessMethodKind.allUserDefinedKinds
                    )
                } content: { model in
                    AccessMethodTypeView(viewModel: model)
                }
            },
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Type", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(title: viewModel.accessMethodViewModel.method.localizedDescription, breadcrumb: nil)
                )

            }
        )
        .padding(.vertical, 24.0)
    }

    private var serverDetailsSection: some View {
        Group {
            if viewModel.accessMethodViewModel.method.hasProxyConfiguration {
                ServerDetailsSection(viewModel: viewModel, focusedInput: $focusedInput)
            } else {
                EmptyView()
            }
        }
    }

    private var testView: some View {
        MethodTestingStatusView(
            status: viewModel.testState,
            onStop: {
                Task {
                    await viewModel.cancelTesting()
                }
            }
        )
        .id(testViewAnchor)
        .padding(.vertical, 24.0)
    }

    private var methodSavingAlert: MullvadAlert {
        MullvadAlert(
            type: .error,
            messages: ["API unreachable, save method anyway?"],
            actions: [
                .init(
                    type: .secondary,
                    title: "Save",
                    identifier: .accessMethodUnreachableSaveButton,
                    handler: {
                        savingAlert = nil
                        viewModel.save()
                        dismiss()
                    }),
                .init(
                    type: .primary,
                    title: "Cancel",
                    identifier: .accessMethodUnreachableCancelButton,
                    handler: {
                        savingAlert = nil
                    }
                ),
            ])
    }

    private var methodDiscardingAlert: MullvadAlert {
        .init(
            type: .warning,
            messages: ["You have unsaved changes."],
            actions: [
                .init(
                    type: .destructive, title: "Discard changes",
                    handler: {
                        viewModel.discard()
                        discardAlert = nil
                        dismiss()
                    }),
                .init(
                    type: .primary, title: "Cancel",
                    handler: {
                        discardAlert = nil
                    }),
            ]
        )
    }

    private func dismissView() {
        focusedInput = nil
        if viewModel.hasChanges {
            discardAlert = methodDiscardingAlert
        } else {
            dismiss()
        }
    }
}
