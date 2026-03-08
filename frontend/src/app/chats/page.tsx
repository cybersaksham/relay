export default function ChatsPage() {
  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">General Chats</h1>
        <p className="mt-2 text-sm text-slate-600">
          Read-only assistant interactions from Slack (Q&amp;A and thread summaries).
        </p>
      </div>

      <div className="surface overflow-hidden">
        <table className="data-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>User</th>
              <th>Mode</th>
              <th>Prompt</th>
              <th>Response</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td colSpan={5} className="px-4 py-6 text-sm text-slate-500">
                General chat history is not exposed by the current backend yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
