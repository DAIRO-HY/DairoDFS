// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "DairoFileSDK",
    platforms: [.iOS(.v13), .macOS(.v10_15)],
    products: [
        .library(name: "DairoFileSDK", targets: ["DairoFileSDK"]),
    ],
    targets: [
        //build.sh生成的静态库
        .binaryTarget(name: "DairoFileSDKFFI", path: "DairoFileSDKFFI.xcframework"),

        //build.sh生成的Swift代码
        .target(
            name: "DairoFileSDK",
            dependencies: ["DairoFileSDKFFI"],
            path: "Sources/DairoFileSDK",
            linkerSettings: [.linkedFramework("Security")]
        ),
        .testTarget(name: "DairoFileSDKTests", dependencies: ["DairoFileSDK"], path: "Tests/DairoFileSDKTests"),
    ]
)
