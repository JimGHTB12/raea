// bevy_feathers' FeathersSelect (select.rs; docs/control.feature FC0a): a menu whose button reads the selected option's
// caption (update_select_caption copies it in; "" when none is selected), its popup a list view of the options at most
// max_visible rows of 28 px high, hidden until it opens. Opening and choosing need input.
import { Children, isValidElement, type ReactElement, type ReactNode } from "react";
import type { NodeProps } from "../ui";
import { px } from "../ui-val";
import { caption } from "./display";
import { FeathersListRow, FeathersListView } from "./listview";
import { FeathersMenu, FeathersMenuButton, FeathersMenuPopup } from "./menu";
import type { RoundedCorners } from "./controls";

/** select.rs's SELECT_ROW_PX: a row's height in the popup's max_height */
const SELECT_ROW_PX = 28;

/** The text inside an element tree, as the caption's Text holds it. */
function textOf(n: ReactNode): string {
  if (typeof n === "string" || typeof n === "number") return String(n);
  if (Array.isArray(n)) return n.map(textOf).join("");
  if (isValidElement(n)) return textOf((n.props as { children?: ReactNode }).children);
  return "";
}

/** FeathersListRows, one per label, captioned, the one at `selected` Selected. @bevy bevy_feathers::controls::list_rows_from_strings */
export function list_rows_from_strings(labels: string[], selected?: number): ReactElement[] {
  if (!Array.isArray(labels) || !labels.every((l) => typeof l === "string")) throw new TypeError(`list_rows_from_strings: labels are strings, got ${JSON.stringify(labels)}`);
  return labels.map((label, i) => <FeathersListRow key={i} selected={i === selected}>{caption(label)}</FeathersListRow>);
}

export interface FeathersSelectProps extends NodeProps {
  /** its options: FeathersListRows, one of them selected (list_rows_from_strings) */
  options: ReactNode;
  corners?: RoundedCorners;
  /** the popup's most rows (8 by default) */
  max_visible?: number;
}

/** A dropdown: a button showing the selected option, its list in a hidden popup. @bevy bevy_feathers::controls::FeathersSelect */
export function FeathersSelect({ options, corners, max_visible = 8, ...node }: FeathersSelectProps) {
  if (!Number.isInteger(max_visible)) throw new TypeError(`FeathersSelect: max_visible is a whole number of rows, got ${max_visible}`);
  const rows = Children.toArray(options);
  const chosen = rows.find((r) => isValidElement(r) && (r.props as { selected?: boolean }).selected);
  return (
    <FeathersMenu {...node}>
      <FeathersMenuButton corners={corners} flex_grow={1}>{caption(chosen ? textOf((chosen as ReactElement<{ children?: ReactNode }>).props.children) : "")}</FeathersMenuButton>
      <FeathersMenuPopup>
        <FeathersListView max_height={px(Math.max(max_visible, 1) * SELECT_ROW_PX)}>{rows}</FeathersListView>
      </FeathersMenuPopup>
    </FeathersMenu>
  );
}
