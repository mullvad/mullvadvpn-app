import SwiftUI

enum MultihopContext: CaseIterable, CustomStringConvertible, Hashable {
    case entry, exit

    var description: String {
        switch self {
        case .entry:
            NSLocalizedString("Entry", comment: "")
        case .exit:
            NSLocalizedString("Exit", comment: "")
        }
    }
}
