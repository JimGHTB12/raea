// window and document first, before the case's libraries evaluate (they check `typeof document` as they load)
import "@raea/ui/dom-globals";
import "parity";
import { mount } from "@raea/ui/react";
import Case from "CASE";
declare const PARITY_FACE: string;
// React DOM's semantics: refs are DOM-like elements, events capture and bubble (docs/ts-native.feature P18); the page's
// "Parity Mono" face registered as Chrome's page has it (build.ts)
mount(
  <>
    <Case />
    <style>{PARITY_FACE}</style>
  </>,
  undefined,
  { dom: true },
);
