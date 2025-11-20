import MullvadTypes
import SwiftUI

struct DeviceManagementView: View {
    enum Style {
        case tooManyDevices((Bool) -> Void)
        case deviceManagement

        var actionButtonTitle: LocalizedStringKey {
            switch self {
            case .deviceManagement:
                return "Remove"
            case .tooManyDevices:
                return "Yes, log out device"
            }
        }

        var actionButtonStyle: MainButtonStyle.Style {
            switch self {
            case .tooManyDevices:
                .danger
            case .deviceManagement:
                .default
            }
        }

        func warningText(deviceName: String) -> [LocalizedStringKey] {
            var attributedDeviceName: AttributedString {
                var fullText = AttributedString(deviceName.capitalized)
                fullText.foregroundColor = Color.mullvadTextPrimary
                return fullText
            }
            return switch self {
            case .tooManyDevices:
                [
                    LocalizedStringKey(
                        "Are you sure you want to log \(attributedDeviceName) out?"
                    )
                ]
            case .deviceManagement:
                [
                    LocalizedStringKey("Remove \(attributedDeviceName)?"),
                    LocalizedStringKey("The device will be removed from the list and logged out."),
                ]
            }
        }
    }

    let deviceManaging: any DeviceManaging
    let style: Style
    let onError: (String, Error) -> Void

    @State private var loggedInDevices: [DeviceListView.Device]?
    @State private var loading = true

    var canLoginNewDevice: Bool {
        guard let loggedInDevices else {
            return false
        }
        return loggedInDevices.count < ApplicationConfiguration.maxAllowedDevices
    }

    var bodyText: LocalizedStringKey {
        switch style {
        case .deviceManagement:
            "View and manage all your logged in devices. You can have up to 5 devices on one account at a time. Each device gets a name when logged in to help you tell them apart easily."
        case .tooManyDevices:
            if canLoginNewDevice {
                "You can now continue logging in on this device."
            } else {
                "Please log out of at least one by removing it from the list below. You can find the corresponding device name under the device’s Account settings."
            }
        }
    }

    private func fetchDevices() {
        loading = true
        _ = deviceManaging.getDevices { result in
            Task { @MainActor in
                loading = false
                switch result {
                case let .success(devices):
                    self.loggedInDevices = devices.map {
                        DeviceListView.Device(
                            id: $0.id,
                            name: $0.name.capitalized,
                            created: $0.created,
                            isCurrentDevice: $0.id == self.deviceManaging.currentDeviceId,
                            isBeingRemoved: false
                        )
                    }
                case let .failure(error):
                    onError("Failed to fetch devices", error)
                }
            }
        }
    }

    @State var deviceManagementAlert: MullvadAlert?
    var body: some View {
        VStack {
            DeviceListView(
                devices: $loggedInDevices,
                loading: $loading,
                onRemoveDevice: { device in
                    deviceManagementAlert = MullvadAlert(
                        type: .warning,
                        messages: style.warningText(deviceName: device.name),
                        action: .init(
                            type: style.actionButtonStyle,
                            title: style.actionButtonTitle,
                            identifier: AccessibilityIdentifier.logOutDeviceConfirmButton,
                            handler: {
                                await withCheckedContinuation { continuation in
                                    guard let loggedInDevices else {
                                        return
                                    }
                                    self.loggedInDevices = loggedInDevices.map {
                                        $0.id == device.id ? $0.setIsBeingRemoved(true) : $0
                                    }
                                    deviceManagementAlert = nil
                                    _ = deviceManaging.deleteDevice(
                                        device.id,
                                        completionHandler: { result in
                                            Task { @MainActor in
                                                switch result {
                                                case .success:
                                                    self.loggedInDevices?.removeAll(where: { $0.id == device.id })
                                                case let .failure(error):
                                                    self.loggedInDevices = loggedInDevices.map {
                                                        $0.id
                                                            == device
                                                            .id ? $0.setIsBeingRemoved(false) : $0
                                                    }
                                                    onError("Failed to log out device", error)
                                                }
                                                continuation.resume()
                                            }
                                        }
                                    )
                                }
                            }
                        ),
                        dismissButtonTitle: "Cancel"
                    )
                },
                header: {
                    AnyView(
                        VStack(alignment: .leading, spacing: 8) {
                            if case .tooManyDevices = style {
                                if canLoginNewDevice {
                                    HStack {
                                        Spacer()
                                        Image.mullvadIconSuccess
                                        Spacer()
                                    }
                                    Text("Super!")
                                        .font(.mullvadBig)
                                        .foregroundStyle(Color.mullvadTextPrimary)
                                } else {
                                    HStack {
                                        Spacer()
                                        Image.mullvadIconFail
                                        Spacer()
                                    }
                                    Text("Too many devices")
                                        .font(.mullvadBig)
                                        .foregroundStyle(Color.mullvadTextPrimary)
                                }
                            }
                            Text(bodyText)
                                .foregroundColor(.mullvadTextPrimary)
                                .opacity(0.6)
                                .font(.mullvadTinySemiBold)
                        })
                }
            )
            Spacer()
            if case let .tooManyDevices(backToLogin) = style {
                MainButton(
                    text: "Continue with login",
                    style: .success
                ) {
                    backToLogin(true)
                }
                .accessibilityIdentifier(AccessibilityIdentifier.continueWithLoginButton)
                .disabled(!canLoginNewDevice)
                .padding(EdgeInsets(UIMetrics.contentLayoutMargins))
            }
        }
        .mullvadAlert(item: $deviceManagementAlert)
        .background(Color.mullvadBackground)
        .task {
            fetchDevices()
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(
            .deviceManagementView
        )
    }
}

#Preview {
    Text("Too many devices")
        .sheet(isPresented: .constant(true)) {
            DeviceManagementView(
                deviceManaging: MockDeviceManaging(),
                style: .tooManyDevices { _ in },
                onError: { _, _ in }
            )
        }
}

#Preview("Too many devices: Success") {
    Text("")
        .sheet(isPresented: .constant(true)) {
            DeviceManagementView(
                deviceManaging: MockDeviceManaging(
                    devicesToReturn: ApplicationConfiguration.maxAllowedDevices - 1
                ),
                style: .tooManyDevices { _ in },
                onError: { _, _ in }
            )
        }
}

#Preview("Device Management") {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                DeviceManagementView(
                    deviceManaging: MockDeviceManaging(),
                    style: .deviceManagement,
                    onError: { _, _ in }
                )
                .navigationTitle("Manage Devices")
            }
        }
}
