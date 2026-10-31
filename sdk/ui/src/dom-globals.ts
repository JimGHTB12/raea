// window and document as globals, before anything else evaluates (docs/ts-native.feature P18): import this first in
// an entry whose libraries check `typeof document` as their modules load (React Aria, Radix).
import { fixNavigator } from "./platform";
import { install } from "./dom";
import { installDnd } from "./dnd";
install();
fixNavigator();
installDnd();
