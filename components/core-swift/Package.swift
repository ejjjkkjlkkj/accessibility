// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AccessibilityCore",
    products: [
        .library(name: "AccessibilityCore", targets: ["AccessibilityCore"])
    ],
    targets: [
        .target(name: "AccessibilityCore"),
        .testTarget(name: "AccessibilityCoreTests", dependencies: ["AccessibilityCore"])
    ]
)
