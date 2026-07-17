import { useCallback, useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play, Star, Pencil, Trash2, FolderPlus, Check, Plus, X, Image, ExternalLink } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

import { AmbientLayer }    from "@/components/AmbientLayer";
import ConfirmDialog        from "@/components/ConfirmDialog";
import {
  getGame, getSessions, launchGame, toggleFavorite, deleteGame,
  getCollections, addGameToCollection, removeGameFromCollection,
  getGalleryImages, storeGalleryImage, deleteGalleryImage,
  getJournalEntries, createJournalEntry,
  getMilestones,
  getRelatedGames, getGameRecommendations,
  type Collection, type AssetRef, type JournalEntry,
  type RelatedGame, type RecommendationResult,
} from "@/lib/api";

import { useGameStoppedListener } from "@/hooks/useGameStoppedListener";
import { formatPlaytime, formatRelativeDate } from "@/lib/utils";
import { useToastStore } from "@/stores/useToastStore";
import type { Game, Session, Milestone } from "@/types";


const ASSET_URL = (p: string) =>
  `https://asset.localhost/${encodeURIComponent(p.replace(/\\/g, "/"))}`;

export default function GameDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { addToast } = useToastStore();

  const [game,         setGame]         = useState<Game | null>(null);
  const [sessions,     setSessions]     = useState<Session[]>([]);
  const [loading,      setLoading]      = useState(true);
  const [launching,    setLaunching]    = useState(false);
  const [error,        setError]        = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [collections,  setCollections]  = useState<Collection[]>([]);
  const [memberIds,    setMemberIds]    = useState<Set<string>>(new Set());
  const [collMenuOpen, setCollMenuOpen] = useState(false);
  const [collLoading,  setCollLoading]  = useState(false);

  // Gallery
  const [gallery,      setGallery]      = useState<AssetRef[]>([]);
  const [lightbox,     setLightbox]     = useState<string | null>(null);
  const [addingImg,    setAddingImg]    = useState(false);

  // Notes
  const [notes,        setNotes]        = useState<JournalEntry[]>([]);
  const [noteBody,     setNoteBody]     = useState("");
  const [savingNote,   setSavingNote]   = useState(false);

  // Related / Recommendations
  const [related,             setRelated]             = useState<RelatedGame[]>([]);
  const [recs,                setRecs]                = useState<RecommendationResult[]>([]);

  // Earned achievements (T47) — milestones in the 'achievement' category
  const [earnedAchievements,  setEarnedAchievements]  = useState<Milestone[]>([]);


  useEffect(() => {
    if (!id) return;
    setLoading(true);
    Promise.allSettled([
      getGame(id), getSessions(id), getCollections(),
      getGalleryImages(id),
      getJournalEntries(id, 10),
      getRelatedGames(id, 8),
      getGameRecommendations(id, 5),
      getMilestones(id, "achievement"),
    ]).then(([g, s, cols, gal, jrn, rel, rec, ach]) => {
      // Core data — propagate error if the game itself can't be loaded.
      if (g.status === "fulfilled") setGame(g.value); else { setError(String(g.reason)); return; }
      if (s.status === "fulfilled") setSessions(s.value);
      if (cols.status === "fulfilled") {
        setCollections(cols.value);
        setMemberIds(new Set(cols.value.filter(c => c.game_ids.includes(id!)).map(c => c.id)));
      }
      // Non-critical sections gracefully degrade to empty.
      if (gal.status === "fulfilled") setGallery(gal.value);
      if (jrn.status === "fulfilled") setNotes(jrn.value);
      if (rel.status === "fulfilled") setRelated(rel.value);
      if (rec.status === "fulfilled") setRecs(rec.value);
      if (ach.status === "fulfilled") setEarnedAchievements(ach.value);
    }).finally(() => setLoading(false));

  }, [id]);

  const handleGameStopped = useCallback((sid: string) => {
    if (sid !== id) return;
    Promise.all([getGame(sid), getSessions(sid)])
      .then(([g, s]) => { setGame(g); setSessions(s); }).catch(() => {});
  }, [id]);
  useGameStoppedListener(handleGameStopped);

  const handleLaunch = async () => {
    if (!game) return;
    setLaunching(true);
    try { await launchGame(game.id); }
    catch (e) { setError(String(e)); }
    finally { setLaunching(false); }
  };

  const handleFavorite = async () => {
    if (!game) return;
    setGame(await toggleFavorite(game.id));
  };

  const handleDelete = async () => {
    if (!game) return;
    try {
      await deleteGame(game.id);
      addToast({ message: `"${game.title}" removed`, type: "success" });
      navigate("/library");
    } catch { addToast({ message: "Failed to delete game", type: "error" }); }
  };

  const handleToggleCollection = async (col: Collection) => {
    if (!game) return;
    setCollLoading(true);
    try {
      if (memberIds.has(col.id)) {
        await removeGameFromCollection(col.id, game.id);
        setMemberIds(prev => { const s = new Set(prev); s.delete(col.id); return s; });
        addToast({ message: `Removed from "${col.name}"`, type: "info" });
      } else {
        await addGameToCollection(col.id, game.id);
        setMemberIds(prev => new Set([...prev, col.id]));
        addToast({ message: `Added to "${col.name}"`, type: "success" });
      }
    } catch { addToast({ message: "Collection update failed", type: "error" }); }
    finally { setCollLoading(false); }
  };

  const handleAddImage = async () => {
    if (!game) return;
    setAddingImg(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png","jpg","jpeg","webp","gif"] }],
      });
      if (!selected || typeof selected !== "string") return;
      const ref = await storeGalleryImage(game.id, selected);
      setGallery(prev => [...prev, ref]);
      addToast({ message: "Image added to gallery", type: "success" });
    } catch (e) { addToast({ message: String(e), type: "error" }); }
    finally { setAddingImg(false); }
  };

  const handleDeleteImage = async (path: string) => {
    try {
      await deleteGalleryImage(path);
      setGallery(prev => prev.filter(g => g.path !== path));
    } catch (e) { addToast({ message: String(e), type: "error" }); }
  };

  const handleAddNote = async () => {
    if (!game || !noteBody.trim()) return;
    setSavingNote(true);
    try {
      const entry = await createJournalEntry({ game_id: game.id, body: noteBody.trim(), entry_type: "note" });
      setNotes(prev => [entry, ...prev]);
      setNoteBody("");
      addToast({ message: "Note saved", type: "success" });
    } catch (e) { addToast({ message: String(e), type: "error" }); }
    finally { setSavingNote(false); }
  };

  if (loading) return (
    <><AmbientLayer coverPath={null} />
      <div style={S.page}><p style={{ color: "var(--color-text-muted)", fontSize: 14 }}>Loading…</p></div></>
  );
  if (error || !game) return (
    <><AmbientLayer coverPath={null} />
      <div style={S.page}>
        <p style={{ color: "var(--color-text-muted)", fontSize: 14 }}>{error ?? "Game not found."}</p>
        <button onClick={() => navigate("/library")} style={S.backBtn}>← Back</button>
      </div></>
  );

  return (
    <>
      <AmbientLayer coverPath={game.cover_path} />
      <div style={S.page}>
        {/* Back */}
        <button onClick={() => navigate(-1)} style={S.backBtn} aria-label="Back">
          <ArrowLeft size={16} /><span>Library</span>
        </button>

        {/* Hero */}
        <div style={S.hero}>
          {game.cover_path
            ? <img src={ASSET_URL(game.cover_path)} alt={game.title} style={S.cover} draggable={false} />
            : <div style={S.coverPlaceholder} />}
          <div style={S.heroMeta}>
            <span style={S.statusTag}>{game.status.toUpperCase()}</span>
            <h1 style={S.title}>{game.title}</h1>
            {(game.developer || game.publisher) && (
              <p style={S.byline}>{[game.developer, game.publisher].filter(Boolean).join(" · ")}</p>
            )}
            {game.genre && <p style={S.genre}>{game.genre}</p>}
            <div style={S.actions}>
              <button onClick={handleLaunch} disabled={launching} style={S.launchBtn} aria-label={`Launch ${game.title}`}>
                <Play size={14} fill="currentColor" />{launching ? "Launching…" : "Play"}
              </button>
              <button onClick={handleFavorite} style={{ ...S.iconBtn, color: game.is_favorite ? "var(--color-text-primary)" : "var(--color-text-disabled)" }} aria-label="Toggle favorite">
                <Star size={16} fill={game.is_favorite ? "currentColor" : "none"} />
              </button>
              <div style={{ position: "relative" }}>
                <button onClick={() => setCollMenuOpen(v => !v)} style={S.iconBtn} aria-label="Add to collection" aria-expanded={collMenuOpen}>
                  <FolderPlus size={15} />
                </button>
                {collMenuOpen && (
                  <div style={S.collMenu} role="listbox">
                    {collections.length === 0 && <p style={S.collMenuEmpty}>No collections yet</p>}
                    {collections.map(col => (
                      <button key={col.id} role="option" aria-selected={memberIds.has(col.id)}
                        onClick={() => handleToggleCollection(col)} disabled={collLoading} style={S.collMenuItem}>
                        {memberIds.has(col.id)
                          ? <Check size={11} style={{ color: "var(--color-text-secondary)", flexShrink: 0 }} />
                          : <span style={{ width: 11, flexShrink: 0 }} />}
                        <span style={S.collMenuLabel}>{col.name}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <button onClick={() => navigate(`/library/${game.id}/edit`)} style={S.iconBtn} aria-label="Edit"><Pencil size={15} /></button>
              <button onClick={() => setConfirmDelete(true)} style={{ ...S.iconBtn, color: "var(--color-text-disabled)" }} aria-label="Delete"><Trash2 size={15} /></button>
            </div>
          </div>
        </div>

        <ConfirmDialog open={confirmDelete} title="Delete game"
          message={`Remove "${game.title}" from your library? This cannot be undone.`}
          confirmLabel="Delete" dangerous onConfirm={handleDelete} onCancel={() => setConfirmDelete(false)} />

        {/* Stats */}
        <div style={S.statsRow}>
          <Stat label="Total playtime" value={formatPlaytime(game.total_playtime_secs)} />
          <Stat label="Sessions"       value={String(sessions.length)} />
          <Stat label="Launches"       value={String(game.launch_count)} />
          {game.last_played && <Stat label="Last played" value={formatRelativeDate(game.last_played)} />}
        </div>

        {/* Gallery */}
        <section style={S.section} aria-labelledby="gallery-heading">
          <div style={S.sectionHeader}>
            <h2 id="gallery-heading" style={S.sectionTitle}>Gallery</h2>
            <button onClick={handleAddImage} disabled={addingImg} style={S.addBtn} aria-label="Add image">
              <Image size={13} />{addingImg ? "Adding…" : "Add Image"}
            </button>
          </div>
          {gallery.length === 0
            ? <p style={S.empty}>No gallery images yet — click "Add Image" to get started.</p>
            : (
              <div style={S.galleryGrid}>
                {gallery.map(img => {
                  const imgPath = typeof img.path === "string" ? img.path : String(img.path);
                  return (
                    <div key={imgPath} style={S.galleryCell}>
                      <img src={ASSET_URL(imgPath)} alt="Gallery" style={S.galleryImg}
                        onClick={() => setLightbox(imgPath)} draggable={false} />
                      <button onClick={() => handleDeleteImage(imgPath)} style={S.galleryDel}
                        aria-label="Delete image" title="Delete">
                        <X size={12} />
                      </button>
                    </div>
                  );
                })}
              </div>
            )}
        </section>

        {/* Lightbox */}
        {lightbox && (
          <div style={S.lightboxBackdrop} onClick={() => setLightbox(null)} role="dialog" aria-label="Image lightbox">
            <button style={S.lightboxClose} onClick={() => setLightbox(null)} aria-label="Close lightbox"><X size={20} /></button>
            <img src={ASSET_URL(lightbox)} alt="Fullscreen" style={S.lightboxImg} onClick={e => e.stopPropagation()} draggable={false} />
          </div>
        )}

        {/* Notes */}
        <section style={S.section} aria-labelledby="notes-heading">
          <div style={S.sectionHeader}>
            <h2 id="notes-heading" style={S.sectionTitle}>Notes</h2>
            <button onClick={() => navigate("/journal")} style={S.linkBtn} aria-label="Open journal">
              <ExternalLink size={12} />Journal
            </button>
          </div>
          <div style={S.noteCompose}>
            <textarea
              value={noteBody}
              onChange={e => setNoteBody(e.target.value)}
              placeholder="Quick note about this game…"
              style={S.noteTextarea}
              rows={3}
              aria-label="New note"
              id="note-input"
            />
            <button onClick={handleAddNote} disabled={savingNote || !noteBody.trim()} style={S.addBtn}>
              <Plus size={13} />{savingNote ? "Saving…" : "Save Note"}
            </button>
          </div>
          {notes.length === 0
            ? <p style={S.empty}>No notes yet.</p>
            : (
              <div style={S.noteList}>
                {notes.map(n => (
                  <div key={n.id} style={S.noteCard}>
                    <p style={S.noteBody}>{n.body}</p>
                    <span style={S.noteMeta}>{formatRelativeDate(n.created_at)}</span>
                  </div>
                ))}
              </div>
            )}
        </section>

        {/* Earned Achievements (T47) — hidden when none earned */}
        {earnedAchievements.length > 0 && (
          <section style={S.section} aria-labelledby="achievements-heading">
            <h2 id="achievements-heading" style={S.sectionTitle}>
              🏆 Earned Achievements ({earnedAchievements.length})
            </h2>
            <ul style={S.achList} aria-label="Earned achievements">
              {earnedAchievements.map((a) => (
                <li key={a.id} style={S.achItem}>
                  <span style={S.achIcon} aria-hidden="true">🏆</span>
                  <div>
                    <p style={S.achName}>{a.title}</p>
                    <p style={S.achMeta}>
                      {formatRelativeDate(a.achievement_date)}
                      {" · "}
                      {a.points} pts
                    </p>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        )}

        {/* Related Titles */}
        {(related.length > 0 || recs.length > 0) && (
          <section style={S.section} aria-labelledby="related-heading">
            <h2 id="related-heading" style={S.sectionTitle}>Related Titles</h2>
            <div style={S.cardStrip}>
              {related.map(r => (
                <button key={r.id} style={S.relCard} onClick={() => navigate(`/library/${r.id}`)}
                  title={`${r.title} — ${r.relation}`}>
                  {r.cover_path
                    ? <img src={ASSET_URL(r.cover_path)} alt={r.title} style={S.relCover} draggable={false} />
                    : <div style={S.relCoverPlaceholder} />}
                  <span style={S.relTitle}>{r.title}</span>
                  <span style={S.relBadge}>{r.relation}</span>
                </button>
              ))}
              {recs.map(r => (
                <button key={r.game_id} style={S.relCard} onClick={() => navigate(`/library/${r.game_id}`)}
                  title={r.reason}>
                  {r.cover_path
                    ? <img src={ASSET_URL(r.cover_path)} alt={r.title} style={S.relCover} draggable={false} />
                    : <div style={S.relCoverPlaceholder} />}
                  <span style={S.relTitle}>{r.title}</span>
                  <span style={{ ...S.relBadge, opacity: 0.6 }}>Suggested</span>
                </button>
              ))}
            </div>
          </section>
        )}

        {/* Sessions */}
        {sessions.length > 0 && (
          <section style={S.section} aria-labelledby="sessions-heading">
            <h2 id="sessions-heading" style={S.sectionTitle}>Recent sessions</h2>
            <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              {sessions.slice(0, 10).map(s => (
                <div key={s.id} style={S.sessionRow}>
                  <span style={{ color: "var(--color-text-muted)", fontSize: 13 }}>{formatRelativeDate(s.started_at)}</span>
                  <span style={{ color: "var(--color-text-primary)", fontFamily: "var(--font-mono)", fontSize: 12 }}>{formatPlaytime(s.duration_secs)}</span>
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <span style={{ fontFamily: "var(--font-display)", fontSize: 28, fontWeight: 600, letterSpacing: "-0.02em", color: "var(--color-text-primary)" }}>{value}</span>
      <span style={{ fontSize: 12, letterSpacing: "0.08em", textTransform: "uppercase" as const, color: "var(--color-text-disabled)", fontFamily: "var(--font-mono)" }}>{label}</span>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  page:              { position: "relative", zIndex: 2, padding: "40px 56px", minHeight: "100vh" },
  backBtn:           { display: "flex", alignItems: "center", gap: 6, background: "none", border: "none", color: "var(--color-text-muted)", fontSize: 13, fontFamily: "var(--font-body)", cursor: "pointer", padding: 0, marginBottom: 40, letterSpacing: "0.04em", textTransform: "uppercase" },
  hero:              { display: "flex", gap: 48, alignItems: "flex-start", marginBottom: 56 },
  cover:             { width: 240, height: 320, objectFit: "cover", flexShrink: 0, borderRadius: 2 },
  coverPlaceholder:  { width: 240, height: 320, flexShrink: 0, background: "var(--color-surface-02)", borderRadius: 2 },
  heroMeta:          { display: "flex", flexDirection: "column", justifyContent: "flex-end", paddingBottom: 8, flex: 1 },
  statusTag:         { fontSize: 11, letterSpacing: "0.12em", color: "var(--color-text-disabled)", fontFamily: "var(--font-mono)", marginBottom: 12 },
  title:             { fontFamily: "var(--font-display)", fontSize: "clamp(40px,5vw,72px)", fontWeight: 700, letterSpacing: "-0.03em", lineHeight: 1.0, color: "var(--color-text-primary)", margin: 0, marginBottom: 12 },
  byline:            { fontSize: 14, color: "var(--color-text-muted)", margin: 0, marginBottom: 6, fontFamily: "var(--font-body)" },
  genre:             { fontSize: 13, color: "var(--color-text-disabled)", margin: 0, marginBottom: 32, fontFamily: "var(--font-mono)", letterSpacing: "0.04em" },
  actions:           { display: "flex", alignItems: "center", gap: 12 },
  launchBtn:         { display: "flex", alignItems: "center", gap: 8, background: "var(--color-text-primary)", color: "var(--color-bg-base)", border: "none", padding: "10px 24px", fontSize: 13, fontFamily: "var(--font-body)", fontWeight: 600, letterSpacing: "0.06em", textTransform: "uppercase", cursor: "pointer", borderRadius: 1, transition: "opacity 150ms" },
  iconBtn:           { background: "none", border: "none", cursor: "pointer", padding: 8, display: "flex", alignItems: "center", color: "var(--color-text-disabled)", transition: "color 150ms" },
  statsRow:          { display: "flex", gap: 48, paddingBottom: 48, borderBottom: "1px solid var(--color-border)", marginBottom: 48 },
  section:           { marginBottom: 56 },
  sectionHeader:     { display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 16 },
  sectionTitle:      { fontFamily: "var(--font-body)", fontSize: 11, fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase", color: "var(--color-text-disabled)", margin: 0 },
  addBtn:            { display: "flex", alignItems: "center", gap: 6, background: "var(--color-surface-02)", border: "1px solid var(--color-border)", color: "var(--color-text-muted)", fontSize: 12, fontFamily: "var(--font-body)", cursor: "pointer", padding: "6px 12px", borderRadius: 1, letterSpacing: "0.04em", transition: "background 150ms" },
  linkBtn:           { display: "flex", alignItems: "center", gap: 5, background: "none", border: "none", color: "var(--color-text-disabled)", fontSize: 12, cursor: "pointer", fontFamily: "var(--font-mono)", letterSpacing: "0.04em" },
  empty:             { color: "var(--color-text-disabled)", fontSize: 13, fontFamily: "var(--font-mono)", letterSpacing: "0.04em", margin: 0 },
  galleryGrid:       { display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 8 },
  galleryCell:       { position: "relative", aspectRatio: "16/9", overflow: "hidden", borderRadius: 2, background: "var(--color-surface-02)", cursor: "pointer" },
  galleryImg:        { width: "100%", height: "100%", objectFit: "cover", transition: "transform 200ms" },
  galleryDel:        { position: "absolute", top: 6, right: 6, background: "rgba(0,0,0,0.7)", border: "none", color: "#fff", borderRadius: 2, width: 24, height: 24, display: "flex", alignItems: "center", justifyContent: "center", cursor: "pointer", opacity: 0 },
  lightboxBackdrop:  { position: "fixed", inset: 0, background: "rgba(0,0,0,0.92)", zIndex: 9999, display: "flex", alignItems: "center", justifyContent: "center" },
  lightboxClose:     { position: "absolute", top: 20, right: 20, background: "none", border: "none", color: "#fff", cursor: "pointer", padding: 8 },
  lightboxImg:       { maxWidth: "90vw", maxHeight: "90vh", objectFit: "contain", borderRadius: 2 },
  noteCompose:       { marginBottom: 16, display: "flex", flexDirection: "column", gap: 8 },
  noteTextarea:      { background: "var(--color-surface-02)", border: "1px solid var(--color-border)", color: "var(--color-text-primary)", fontFamily: "var(--font-body)", fontSize: 13, padding: "10px 12px", borderRadius: 2, resize: "vertical", outline: "none", lineHeight: 1.6 },
  noteList:          { display: "flex", flexDirection: "column", gap: 8 },
  noteCard:          { background: "var(--color-surface-02)", border: "1px solid var(--color-border)", borderRadius: 2, padding: "12px 14px" },
  noteBody:          { color: "var(--color-text-primary)", fontSize: 13, fontFamily: "var(--font-body)", margin: 0, marginBottom: 8, lineHeight: 1.6 },
  noteMeta:          { color: "var(--color-text-disabled)", fontSize: 11, fontFamily: "var(--font-mono)", letterSpacing: "0.04em" },
  cardStrip:         { display: "flex", gap: 12, overflowX: "auto", paddingBottom: 8 },
  relCard:           { display: "flex", flexDirection: "column", gap: 6, background: "var(--color-surface-02)", border: "1px solid var(--color-border)", borderRadius: 2, padding: 10, cursor: "pointer", flexShrink: 0, width: 140, textAlign: "left", transition: "border-color 150ms" },
  relCover:          { width: "100%", aspectRatio: "3/4", objectFit: "cover", borderRadius: 1 },
  relCoverPlaceholder: { width: "100%", aspectRatio: "3/4", background: "var(--color-surface)", borderRadius: 1 },
  relTitle:          { fontSize: 12, fontFamily: "var(--font-body)", color: "var(--color-text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" },
  relBadge:          { fontSize: 10, fontFamily: "var(--font-mono)", letterSpacing: "0.06em", color: "var(--color-text-disabled)", textTransform: "uppercase" },
  sessionRow:        { display: "flex", justifyContent: "space-between", padding: "10px 0", borderBottom: "1px solid var(--color-border)" },
  collMenu:          { position: "absolute", top: "calc(100% + 8px)", left: "50%", transform: "translateX(-50%)", background: "var(--color-surface)", border: "1px solid var(--color-border)", borderRadius: 1, minWidth: 180, zIndex: 50, boxShadow: "0 8px 24px rgba(0,0,0,0.4)", padding: "6px 0", display: "flex", flexDirection: "column" },
  collMenuEmpty:     { fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--color-text-disabled)", padding: "8px 14px", margin: 0, letterSpacing: "0.06em" },
  collMenuItem:      { display: "flex", alignItems: "center", gap: 8, background: "none", border: "none", padding: "8px 14px", cursor: "pointer", width: "100%", textAlign: "left", transition: "background 100ms" },
  collMenuLabel:     { fontFamily: "var(--font-body)", fontSize: 13, color: "var(--color-text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" },

  // Earned Achievements panel (T47)
  achList:  { listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column" as const, gap: 12 },
  achItem:  { display: "flex", alignItems: "flex-start", gap: 12, padding: "10px 14px", background: "var(--color-surface-02)", border: "1px solid var(--color-border)", borderRadius: 2 },
  achIcon:  { fontSize: 20, lineHeight: 1, flexShrink: 0, marginTop: 2 },
  achName:  { fontFamily: "var(--font-body)", fontSize: 13, fontWeight: 600, color: "var(--color-text-primary)", margin: 0, marginBottom: 4 },
  achMeta:  { fontFamily: "var(--font-mono)", fontSize: 11, letterSpacing: "0.04em", color: "var(--color-text-disabled)", margin: 0 },};
