import Foundation
import Security

struct PeerforaCredentialStore {
    private let serviceName: String

    init(serviceName: String = Bundle.main.bundleIdentifier ?? "com.peerfora.student") {
        self.serviceName = serviceName
    }

    func saveSessionToken(_ sessionToken: String, accountIdentifier: String) throws {
        let sessionTokenData = Data(sessionToken.utf8)
        let existingQuery = credentialQuery(for: accountIdentifier)
        let deleteStatus = SecItemDelete(existingQuery as CFDictionary)
        guard deleteStatus == errSecSuccess || deleteStatus == errSecItemNotFound else {
            throw PeerforaCredentialStoreError.unableToDelete(deleteStatus)
        }

        var newCredential = existingQuery
        newCredential[kSecValueData as String] = sessionTokenData
        newCredential[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly

        let status = SecItemAdd(newCredential as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw PeerforaCredentialStoreError.unableToSave(status)
        }
    }

    func readSessionToken(accountIdentifier: String) throws -> String? {
        var query = credentialQuery(for: accountIdentifier)
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var storedCredential: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &storedCredential)
        guard status != errSecItemNotFound else {
            return nil
        }
        guard status == errSecSuccess, let sessionTokenData = storedCredential as? Data else {
            throw PeerforaCredentialStoreError.unableToRead(status)
        }

        return String(data: sessionTokenData, encoding: .utf8)
    }

    func deleteSessionToken(accountIdentifier: String) throws {
        let status = SecItemDelete(credentialQuery(for: accountIdentifier) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw PeerforaCredentialStoreError.unableToDelete(status)
        }
    }

    private func credentialQuery(for accountIdentifier: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: accountIdentifier,
        ]
    }
}

enum PeerforaCredentialStoreError: Error {
    case unableToSave(OSStatus)
    case unableToRead(OSStatus)
    case unableToDelete(OSStatus)
}
