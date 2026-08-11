import type { Change, Relationship } from "./api";

const labels: Record<string, string> = {
  followers: "Followers",
  following: "Following",
  mutuals: "Mutuals",
  not_following_back: "Not following back",
  followers_not_followed_back: "Followers you do not follow",
};

type Props = {
  mode: "lists" | "changes";
  hasPreviousSnapshot: boolean;
  relationships: Relationship[];
  changes: Change[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  onLoadMore: () => void;
  onSelect?: (username: string) => void;
};

export function ResultsTable({
  mode,
  hasPreviousSnapshot,
  relationships,
  changes,
  hasNextPage,
  isFetchingNextPage,
  onLoadMore,
  onSelect,
}: Props) {
  if (mode === "changes" && !hasPreviousSnapshot) {
    return (
      <div className="table">
        <div className="thead"><span>Account</span><span>Change</span></div>
        <div className="empty">
          This is the oldest snapshot. Select a newer snapshot to compare it with the immediately prior import.
        </div>
      </div>
    );
  }

  const rows = mode === "changes" ? changes : relationships;
  return (
    <div className="table">
      <div className="thead"><span>Account</span><span>{mode === "changes" ? "Change" : "Relationship"}</span></div>
      {rows.map((row) => {
        const direction = "direction" in row ? row.direction : undefined;
        const label = direction ?? labels[(row as Relationship).kind] ?? (row as Relationship).kind;
        return (
          <div className={`tr ${onSelect ? "selectable" : ""}`} key={`${row.username}-${direction ?? ""}`} onClick={()=>onSelect?.(row.username)}>
            <div className="account">
              <div className="avatar">{row.username.slice(0, 2).toUpperCase()}</div>
              <div>
                <strong>@{row.username}</strong>
                {row.profileUrl && <span>Instagram profile</span>}
              </div>
            </div>
            <span className={`badge ${direction ?? ""}`}>{label}</span>
          </div>
        );
      })}
      {rows.length === 0 && <div className="empty">No matching accounts.</div>}
      {hasNextPage && (
        <button className="load-more" onClick={onLoadMore} disabled={isFetchingNextPage}>
          {isFetchingNextPage ? "Loading…" : "Load more"}
        </button>
      )}
    </div>
  );
}
