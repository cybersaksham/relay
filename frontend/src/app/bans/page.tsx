export default function BansPage() {
  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Bans and Strikes</h1>
      </div>

      <div className="surface overflow-hidden">
        <table className="data-table">
          <thead>
            <tr>
              <th>Slack User ID</th>
              <th>Active Ban Until</th>
              <th>Strikes (24h)</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td colSpan={4} className="px-4 py-6 text-sm text-slate-500">
                Ban management is not exposed by the current backend yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
