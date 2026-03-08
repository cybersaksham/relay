export default function SlackPage() {
  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Slack Request Audit</h1>
      </div>

      <div className="surface overflow-hidden">
        <div className="overflow-x-auto">
          <table className="data-table">
            <thead>
              <tr>
                <th>Time</th>
                <th>User</th>
                <th>Event</th>
                <th>Lane</th>
                <th>Policy</th>
                <th>Environment</th>
                <th>Workflow</th>
                <th>Route</th>
                <th>Context</th>
                <th>Request Preview</th>
                <th>Decision</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td colSpan={12} className="px-4 py-6 text-sm text-slate-500">
                  Slack audit history is not exposed by the current backend yet.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
