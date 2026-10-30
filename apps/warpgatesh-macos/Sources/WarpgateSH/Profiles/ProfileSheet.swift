enum ProfileSheet: Identifiable {
  case new
  case renew(CompanionProfile)
  var id: String {
    switch self {
    case .new: "new"
    case .renew(let profile): "renew-" + profile.name
    }
  }
}
