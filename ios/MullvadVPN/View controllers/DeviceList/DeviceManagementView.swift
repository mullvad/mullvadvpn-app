import MullvadTypes
import SwiftUI

struct DeviceManagementView: View {
    enum Style {
        case tooManyDevices((Bool) -> Void)
        case normal
    }

    let deviceManaging: any DeviceManaging
    let style: Style
    let onError: (String, Error) -> Void

    @State private var devices: [DeviceListView.Device] = []
    @State private var loading = true

    var canLogin: Bool {
        devices.count < ApplicationConfiguration.maxAllowedDevices
    }

    var bodyText: LocalizedStringKey {
        switch style {
        case .normal:
            """
            View and manage all your logged in devices.\
            You can have up to 5 devices on one account at a time.\
            Each device gets a name when logged in to help you tell them apart easily.
            """
        case .tooManyDevices:
            """
            Please log out of at least one by removing it from the list below.\
            You can find the corresponding device name under the device’s Account settings.
            """
        }
    }

    private func fetchDevices() {
        loading = true
        _ = deviceManaging.getDevices { result in
            Task { @MainActor in
                loading = false
                switch result {
                case let .success(devices):
                    self.devices = devices.map {
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
            if case .tooManyDevices = style {
                VStack(alignment: .leading, spacing: 8) {
                    if devices.count < ApplicationConfiguration.maxAllowedDevices {
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
                .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
            }
            HStack {
                Text(bodyText)
                    .foregroundColor(.mullvadTextPrimary)
                    .opacity(0.6)
                    .font(.mullvadTinySemiBold)
                Spacer()
            }
            .padding(EdgeInsets(top: 8, leading: 16, bottom: 16, trailing: 16))
            DeviceListView(
                devices: $devices,
                loading: $loading,
                onRemoveDevice: { device in
                    var deviceName: AttributedString {
                        var fullText = AttributedString(device.name.capitalized)
                        fullText.foregroundColor = Color.mullvadTextPrimary
                        return fullText
                    }
                    deviceManagementAlert = MullvadAlert(
                        type: .warning,
                        message: LocalizedStringKey("""
                            Remove \(deviceName)?
                            The device will be removed from the list and logged out.
                            """
                        ),
                        action: .init(
                            type: .default,
                            title: "Remove",
                            handler: {
                                await withCheckedContinuation { continuation in
                                    devices = devices.map {
                                        $0.id == device
                                            .id ? $0.setIsBeingRemoved(true) : $0
                                    }
                                    deviceManagementAlert = nil
                                    _ = deviceManaging.deleteDevice(
                                        device.id,
                                        completionHandler: { result in
                                            Task { @MainActor in
                                                switch result {
                                                case .success:
                                                    devices.removeAll(where: { $0.id == device.id })
                                                case let .failure(error):
                                                    devices = devices.map {
                                                        $0.id == device
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
                .disabled(!canLogin)
                .padding()
            }
        }
        .mullvadAlert(item: $deviceManagementAlert)
        .background(Color.mullvadBackground)
        .task {
            fetchDevices()
        }
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
                    style: .normal,
                    onError: { _, _ in }
                )
                .navigationTitle("Manage Devices")
            }
        }
}
