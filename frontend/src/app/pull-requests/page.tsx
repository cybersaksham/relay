export default function PullRequestsPage() {
  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Pull Requests</h1>
      </div>

      <div className="surface overflow-hidden">
        <table className="data-table">
          <thead>
            <tr>
              <th>Task</th>
              <th>Repo</th>
              <th>Branch</th>
              <th>Status</th>
              <th>PR</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td colSpan={6} className="px-4 py-6 text-sm text-slate-500">
                Pull request tracking is not exposed by the current backend yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
