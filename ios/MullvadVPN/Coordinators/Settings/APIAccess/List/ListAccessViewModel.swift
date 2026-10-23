import Combine
import MullvadLogging
import MullvadSettings
import MullvadTypes
import SwiftUI

enum AccessMethodStatus {
    case off
    case inUse
    case warning(String)
    case error(String)

    var description: String {
        switch self {
        case .off:
            NSLocalizedString("Off", comment: "")
        case .inUse:
            NSLocalizedString("In use", comment: "")
        case .warning(let message), .error(let message):
            NSLocalizedString(message, comment: "")
        }
    }
}

@MainActor
protocol ListAccessViewModelProtocol: AnyObject {
    var items: [ListAccessMethodItem] { get }
    var infoHeaderConfig: InfoHeaderConfig { get }
    var infoModalConfig: InfoModalConfig { get }

    func getStatus(for item: ListAccessMethodItem) -> AccessMethodStatus?
    func fetch(_ item: ListAccessMethodItem) -> PersistentAccessMethod?
}

@MainActor
@Observable
final class ListAccessViewModel: ListAccessViewModelProtocol {
    private let interactor: ListAccessMethodInteractorProtocol
    private var cancellables = Set<Combine.AnyCancellable>()
    private var itemInUse: ListAccessMethodItem?

    private(set) var items: [ListAccessMethodItem] = []
    let logger = Logger(label: "ListAccessViewModel")

    init(interactor: ListAccessMethodInteractorProtocol) {
        self.interactor = interactor

        interactor.itemsPublisher
            .receive(on: DispatchQueue.main)
            .sink { [weak self] items in
                self?.items = items
            }
            .store(in: &cancellables)

        interactor.itemInUsePublisher
            .receive(on: DispatchQueue.main)
            .sink { [weak self] item in
                self?.itemInUse = item
            }
            .store(in: &cancellables)
    }

    var infoHeaderConfig: InfoHeaderConfig {
        InfoHeaderConfig(
            body: NSLocalizedString(
                "Manage and add custom methods to access the Mullvad API.",
                comment: ""
            ),
            link: NSLocalizedString("About API access…", comment: "")
        )
    }

    var infoModalConfig: InfoModalConfig {
        InfoModalConfig(
            header: NSLocalizedString("API access", comment: ""),
            preamble: NSLocalizedString(
                "Manage and add custom methods to access the Mullvad API.",
                comment: ""
            ),
            body: [
                NSLocalizedString(
                    "The app needs to communicate with a Mullvad API server to log you in, "
                        + "fetch server lists, and other critical operations.",
                    comment: ""
                ),
                NSLocalizedString(
                    "On some networks, where various types of censorship are being used, "
                        + "the API servers might not be directly reachable.",
                    comment: ""
                ),
                NSLocalizedString(
                    "This feature allows you to circumvent that censorship by adding custom ways "
                        + "to access the API via proxies and similar methods.",
                    comment: ""
                ),
            ]
        )
    }

    // MARK: - Public methods

    func getStatus(for item: ListAccessMethodItem) -> AccessMethodStatus? {
        guard cipherIsValid(for: item) else {
            return .warning("Unsupported cipher")
        }

        if itemInUse?.id == item.id {
            return .inUse
        }

        return item.isEnabled ? nil : .off
    }

    func fetch(_ item: ListAccessMethodItem) -> PersistentAccessMethod? {
        guard let item = interactor.accessMethod(by: item.id) else {
            logger.error("Failed to create view model for access method: \(item.name).")
            return nil
        }
        return item
    }

    // MARK: - Private methods

    private func cipherIsValid(for item: ListAccessMethodItem) -> Bool {
        guard
            case let .shadowsocks(configuration)? =
                interactor.accessMethod(by: item.id)?.proxyConfiguration
        else {
            return true
        }

        return interactor.shadowsocksCiphers.contains(configuration.cipher)
    }
}
