// The platform a React app leans on beyond the DOM (docs/ts-native.feature P17), for the engines raea-runtime-js runs a bundle
// on (JavaScriptCore's bare JSContext and QuickJS have the language and Intl, and none of the web's APIs): each global
// is installed only where the engine has none, so a host that has the real one (bun in tests, a browser) keeps it.
//
//   pure JS, as Chrome behaves:  DOMException, Event/EventTarget (until the DOM layer's replace them), AbortController
//     and AbortSignal (abort, timeout, any), TextEncoder/TextDecoder, atob/btoa, URL/URLSearchParams, structuredClone,
//     Blob/File/FileReader, FormData, Headers/Request/Response, ReadableStream (what a Response's body needs),
//     Storage (localStorage, sessionStorage), history and location (in memory), crypto.getRandomValues/randomUUID,
//     performance marks and measures, the console's rest, reportError, setInterval's arguments
//   through the host (raea-runtime-js's built-in calls, runtime/js/src/net.rs):  fetch over http(s) ("net.fetch"),
//     WebSocket ("net.ws.*", its events as the host event "net.ws"), localStorage's persistence ("net.storage.*"),
//     the system clipboard ("net.clipboard.*"), Web Audio's decodeAudioData ("audio.decode", into an AudioBuffer of an
//     OfflineAudioContext). data: and blob: URLs never leave JS.
//
// Where a host answers none of those (a test, a host without the calls), fetch of http(s) rejects with a TypeError as
// a browser offline does, WebSocket errors and closes (1006), storage stays in memory and the clipboard is this
// process's own.
export { clipboard, fixNavigator } from "./platform/all";


