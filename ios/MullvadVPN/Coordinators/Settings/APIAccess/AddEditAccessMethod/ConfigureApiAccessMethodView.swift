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

struct ConfigureApiAccessMethodView: View {
    @State var viewModel: any ConfigureApiAccessMethodViewModelProtocol
    @Environment(\.dismiss) private var dismiss
    @State private var savingAlert: MullvadAlert?
    @FocusState private var focusedInput: AccessMethodField?
    @State private var hasAppeared: Bool = false
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
            .scrollBounceBehavior(.basedOnSize)
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
        .onAppear {
            LocalNetworkProbe()
                .triggerLocalNetworkPrivacyAlert()
        }
        .onDisappear {
            Task {
                focusedInput = nil
                await viewModel.cancelTesting()
            }
        }
        .doneBarButton(
            accessibilityIdentifier: .accessMethodAddButton,
            isDisabled: !viewModel.canSubmit
        ) {
            focusedInput = nil
            Task {
                await viewModel.startTesting()
            }
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
        Group {
            SegmentedListItem(
                isLastInList: true,
                userInteraction: .enabledWithoutHighlight,
                accessibilityIdentifier: .accessMethodNameTextField,
                accessibilityLabel: NSLocalizedString("Name", comment: ""),
                borderStyle: .constant(viewModel.borderStyle(for: .name)),
                leading: {
                    AccessMethodLabelView(
                        title: "Name",
                        textColor: viewModel.borderStyle(for: .name).textColor
                    )
                },
                trailing: {
                    AccessMethodTextField(
                        placeholder: "Required",
                        text: $viewModel.accessMethodViewModel.name,
                        field: .name,
                        focusedField: $focusedInput
                    )
                    .onAppear {
                        guard !hasAppeared else { return }
                        hasAppeared = true
                        focusedInput = .name
                    }
                }
            )

            if let error = viewModel.hasError(for: .name) {
                MessageView(
                    message: .init(
                        text: "\(error.errorDescription)",
                        appearance: .error
                    )
                )
                .padding(.leading, 16.0)
            }
        }
        .onChange(of: viewModel.accessMethodViewModel.name) {
            viewModel.clearErrors(for: [.name])
        }
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
                        .onDisappear {
                            focusedInput = .server
                        }
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
}
