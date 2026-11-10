import SwiftUI

// RR0's film on the iPhone (docs/rerender.feature, bench/rerender/ios/bench.sh). One launch renders one composition,
// so each runs in a cold process as bench.sh renders each chunk on the Mac:
//   RR_COMP=NAME  (launch environment) Bundle/NAME.comp.js rendered to Documents/NAME.mp4
// What raea prints (its startup marks, RAEA_TIMES' line, APPLIED or REFUSED) goes to Documents/NAME.log, the frame log
// to Documents/NAME.tsv, and Documents/NAME.json says the wall time and the peak physical footprint (what jetsam
// counts). The app exits when it's done, which is what bench.sh's `devicectl ... launch --console` waits on.

/// The process's peak physical footprint, in MB.
func peakMB() -> Int {
    var info = task_vm_info_data_t()
    var count = mach_msg_type_number_t(MemoryLayout<task_vm_info_data_t>.size / MemoryLayout<natural_t>.size)
    let kr = withUnsafeMutablePointer(to: &info) {
        $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { task_info(mach_task_self_, task_flavor_t(TASK_VM_INFO), $0, &count) }
    }
    return kr == KERN_SUCCESS ? Int(info.ledger_phys_footprint_peak >> 20) : -1
}

@main
struct RerenderBenchApp: App {
    @State private var text = "rendering…"
    var body: some Scene {
        WindowGroup {
            Text(text).font(.system(size: 13, design: .monospaced)).padding()
                .task { text = await Self.run() }
        }
    }

    static func run() async -> String {
        guard let name = ProcessInfo.processInfo.environment["RR_COMP"] else { return "no RR_COMP: bench.sh launches this" }
        let bundleDir = Bundle.main.resourceURL!.appendingPathComponent("Bundle")
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        let js = bundleDir.appendingPathComponent(name + ".comp.js").path
        let out = docs.appendingPathComponent(name + ".mp4")
        let log = docs.appendingPathComponent(name + ".log").path
        let report = docs.appendingPathComponent(name + ".json")
        for f in [out, report] { try? FileManager.default.removeItem(at: f) }
        FileManager.default.changeCurrentDirectoryPath(docs.path)
        // as bench.sh renders on the Mac: no signing, no mark, no export cache, RAEA_TIMES' line, the frame log
        setenv("RAEA_SIGN", "0", 1); setenv("RAEA_MARK", "0", 1); setenv("RAEA_EXPORT_CACHE", "0", 1)
        setenv("RAEA_TIMES", "1", 1)
        setenv("RAEA_FRAME_LOG", docs.appendingPathComponent(name + ".tsv").path, 1)
        // iOS gives an app no JIT; said to JavaScriptCore too, before its first context
        setenv("JSC_useJIT", "false", 1)
        // a locked phone suspends the app and the run is lost
        await MainActor.run { UIApplication.shared.isIdleTimerDisabled = true }
        let said: String = await Task.detached {
            freopen(log, "w", stdout); freopen(log, "a", stderr)
            setvbuf(stdout, nil, _IOLBF, 0)
            let t0 = Date()
            let code = raea_rerender_run(js, out.path)
            let wall = Date().timeIntervalSince(t0)
            fflush(stdout); fflush(stderr)
            let body = "{\"comp\": \"\(name)\", \"code\": \(code), \"wall_s\": \(String(format: "%.3f", wall)), \"peak_mb\": \(peakMB())}"
            try? body.write(to: report, atomically: true, encoding: .utf8)
            return body
        }.value
        exit(0)
        return said
    }
}
