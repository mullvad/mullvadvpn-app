public struct StoreKitTransaction: Codable, Sendable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}
