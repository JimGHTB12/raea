// Lists and tables: a ul and an ol of listitems (a nested list's items at level 2), and a table of rows,
// columnheaders and cells (each named by its text).
// proves: aria.role.list, aria.role.listitem, aria.role.table, aria.role.row, aria.role.columnheader, aria.role.cell, aria.html-aam.el-ul, aria.html-aam.el-ol, aria.html-aam.el-li, aria.html-aam.el-table, aria.html-aam.el-tr, aria.html-aam.el-th, aria.html-aam.el-td
export const steps = [];
const f = "block h-6 w-40";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="opacity-0">
        <ul data-testid="ul" className="block w-40">
          <li data-testid="li-a" className={f}>Apples</li>
          <li data-testid="li-b" className="block w-40">
            Pears
            <ol data-testid="ol" className="block w-40">
              <li data-testid="li-nested" className={f}>Conference</li>
            </ol>
          </li>
        </ul>
        <table data-testid="table" className="w-60">
          <thead>
            <tr data-testid="head-row">
              <th data-testid="th" className="h-6">Name</th>
              <th className="h-6">Qty</th>
            </tr>
          </thead>
          <tbody>
            <tr data-testid="row">
              <td data-testid="td" className="h-6">Apple</td>
              <td className="h-6">3</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
