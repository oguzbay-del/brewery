import Foundation

/// What the code points to.
enum ScanKind: String, Sendable, Equatable {
    case container
    case vessel
    case lot
}

/// An action the server lists for the caller in `ScanResolution.actions`.
/// The interface shows only the listed actions; the server checks every action again.
enum ScanAction: String, CaseIterable, Sendable, Equatable {
    case view
    case changeStatus = "change_status"
    case fill
    case retire
    case recall
    case printLabel = "print_label"

    /// Catalog key of the action text.
    var textKey: String { "scan.action_" + rawValue }

    /// `view` reads data; every other action writes data and needs a writable module.
    var isWrite: Bool { self != .view }
}

/// One label and value row on the card. `labelKey` is a catalog key.
struct ScanDetail: Sendable, Equatable {
    let labelKey: String
    let value: String
}

/// The card for a scanned code: the object and only the permitted actions.
struct ScanCard: Sendable, Equatable {
    let kind: ScanKind
    let id: UUID
    let label: String
    let details: [ScanDetail]
    let actions: [ScanAction]
}

/// Fields of `ScanResolution` the card needs. The response is read through these types
/// so the mapping does not depend on the names the generator makes for nullable objects.
struct ScanResolutionDTO: Decodable {
    struct ContainerDTO: Decodable {
        let serialNumber: String
        let status: String
        let packageTypeName: String
        let volumeL: Double
    }

    struct VesselDTO: Decodable {
        let code: String
        let name: String
        let capacityL: Double
        let cleaningStatus: String
    }

    struct LotDTO: Decodable {
        let lotCode: String
        let ingredientName: String
        let unit: String
        let quantity: Double
        let expiresOn: String?
    }

    let kind: String
    let id: UUID
    let label: String
    let actions: [String]
    let container: ContainerDTO?
    let vessel: VesselDTO?
    let lot: LotDTO?
}

extension ScanCard {
    /// Builds the card. Returns `nil` for a kind this app version does not know.
    /// Unknown action names are dropped: the app cannot run them.
    init?(_ dto: ScanResolutionDTO, locale: KatalogLocale = .device) {
        guard let kind = ScanKind(rawValue: dto.kind) else { return nil }
        self.kind = kind
        id = dto.id
        label = dto.label
        actions = dto.actions.compactMap(ScanAction.init(rawValue:))
        details = Self.details(dto, locale: locale)
    }

    private static func details(_ dto: ScanResolutionDTO, locale: KatalogLocale) -> [ScanDetail] {
        if let container = dto.container {
            let statusKey = "containers.status_" + container.status
            let status = UiText.text(statusKey, locale: locale, fallback: container.status)
            return [
                ScanDetail(labelKey: "scan.detail_serial", value: container.serialNumber),
                ScanDetail(labelKey: "scan.detail_status", value: status),
                ScanDetail(labelKey: "scan.detail_package", value: container.packageTypeName),
                ScanDetail(
                    labelKey: "scan.detail_volume",
                    value: UiText.quantity(container.volumeL, unitCode: "l", locale: locale)
                ),
            ]
        }
        if let vessel = dto.vessel {
            return [
                ScanDetail(labelKey: "scan.detail_code", value: vessel.code),
                ScanDetail(labelKey: "scan.detail_name", value: vessel.name),
                ScanDetail(
                    labelKey: "scan.detail_capacity",
                    value: UiText.quantity(vessel.capacityL, unitCode: "l", locale: locale)
                ),
                ScanDetail(labelKey: "scan.detail_cleaning", value: vessel.cleaningStatus),
            ]
        }
        if let lot = dto.lot {
            let quantity = UiText.quantity(lot.quantity, unitCode: lot.unit, locale: locale)
            var rows = [
                ScanDetail(labelKey: "scan.detail_code", value: lot.lotCode),
                ScanDetail(labelKey: "scan.detail_ingredient", value: lot.ingredientName),
                ScanDetail(labelKey: "scan.detail_quantity", value: quantity),
            ]
            if let expires = lot.expiresOn {
                rows.append(ScanDetail(labelKey: "scan.detail_expires", value: String(expires.prefix(10))))
            }
            return rows
        }
        return []
    }
}
