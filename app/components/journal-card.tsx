"use client";

import { useState, useCallback } from "react";
import { toast } from "sonner";
import { useWallet } from "../lib/wallet/context";
import { useSendTransaction } from "../lib/hooks/use-send-transaction";
import {
  useJournalEntries,
  type JournalEntry,
} from "../lib/hooks/use-journal-entries";
import {
  getCreateJournalEntryInstructionAsync,
  getUpdateJournalEntryInstructionAsync,
  getDeleteJournalEntryInstructionAsync,
} from "../generated/crud";
import { parseTransactionError } from "../lib/errors";
import { useCluster } from "./cluster-context";
import { ellipsify } from "../lib/explorer";

const TITLE_MAX = 32;
const MESSAGE_MAX = 128;

export function JournalCard() {
  const { wallet, signer, status } = useWallet();
  const { send, isSending } = useSendTransaction();
  const { getExplorerUrl } = useCluster();
  const { entries, isLoading, mutate } = useJournalEntries();

  const [showCreate, setShowCreate] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newMessage, setNewMessage] = useState("");
  const [editingAddress, setEditingAddress] = useState<string | null>(null);
  const [editMessage, setEditMessage] = useState("");
  const [deletingAddress, setDeletingAddress] = useState<string | null>(null);

  const handleCreate = useCallback(async () => {
    if (!signer || !newTitle.trim() || !newMessage.trim()) return;

    try {
      const ix = await getCreateJournalEntryInstructionAsync({
        owner: signer,
        title: newTitle.trim(),
        message: newMessage.trim(),
      });
      const sig = await send({ instructions: [ix] });
      await mutate();
      toast.success("Entry created!", {
        description: (
          <a
            href={getExplorerUrl(`/tx/${sig}`)}
            target="_blank"
            rel="noopener noreferrer"
            className="underline"
          >
            View transaction
          </a>
        ),
      });
      setNewTitle("");
      setNewMessage("");
      setShowCreate(false);
    } catch (err) {
      toast.error(parseTransactionError(err));
    }
  }, [signer, newTitle, newMessage, send, mutate, getExplorerUrl]);

  const handleUpdate = useCallback(
    async (entry: JournalEntry) => {
      if (!signer || !editMessage.trim()) return;

      try {
        const ix = await getUpdateJournalEntryInstructionAsync({
          owner: signer,
          title: entry.title,
          message: editMessage.trim(),
        });
        const sig = await send({ instructions: [ix] });
        await mutate();
        toast.success("Entry updated!", {
          description: (
            <a
              href={getExplorerUrl(`/tx/${sig}`)}
              target="_blank"
              rel="noopener noreferrer"
              className="underline"
            >
              View transaction
            </a>
          ),
        });
        setEditingAddress(null);
      } catch (err) {
        toast.error(parseTransactionError(err));
      }
    },
    [signer, editMessage, send, mutate, getExplorerUrl],
  );

  const handleDelete = useCallback(
    async (entry: JournalEntry) => {
      if (!signer) return;

      try {
        const ix = await getDeleteJournalEntryInstructionAsync({
          owner: signer,
          title: entry.title,
        });
        const sig = await send({ instructions: [ix] });
        await mutate();
        toast.success("Entry deleted!", {
          description: (
            <a
              href={getExplorerUrl(`/tx/${sig}`)}
              target="_blank"
              rel="noopener noreferrer"
              className="underline"
            >
              View transaction
            </a>
          ),
        });
        setDeletingAddress(null);
      } catch (err) {
        toast.error(parseTransactionError(err));
      }
    },
    [signer, send, mutate, getExplorerUrl],
  );

  if (status !== "connected" || !wallet) {
    return (
      <section className="w-full space-y-4 rounded-2xl border border-border-low bg-card p-6">
        <div className="space-y-1">
          <p className="text-lg font-semibold">Journal Entries</p>
          <p className="text-sm text-muted">
            Connect your wallet to manage your on-chain journal.
          </p>
        </div>
        <div className="rounded-lg bg-cream/50 p-4 text-center text-sm text-muted">
          Wallet not connected
        </div>
      </section>
    );
  }

  return (
    <section className="w-full space-y-4 rounded-2xl border border-border-low bg-card p-6 shadow-[0_20px_80px_-50px_rgba(0,0,0,0.35)]">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="space-y-0.5">
          <p className="text-lg font-semibold">Journal Entries</p>
          <p className="text-sm text-muted">
            {entries.length} {entries.length === 1 ? "entry" : "entries"} on-chain
          </p>
        </div>
        {!showCreate && (
          <button
            onClick={() => setShowCreate(true)}
            disabled={isSending}
            className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
          >
            + New Entry
          </button>
        )}
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="space-y-3 rounded-xl border border-border-low bg-cream/20 p-4">
          <p className="text-sm font-medium">New Entry</p>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-xs text-muted">Title</label>
              <span className="text-xs text-muted">
                {newTitle.length}/{TITLE_MAX}
              </span>
            </div>
            <input
              type="text"
              maxLength={TITLE_MAX}
              placeholder="Entry title"
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              disabled={isSending}
              className="w-full rounded-lg border border-border-low bg-card px-3 py-2 text-sm outline-none transition placeholder:text-muted focus:border-foreground/30 disabled:opacity-50"
            />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-xs text-muted">Message</label>
              <span className="text-xs text-muted">
                {newMessage.length}/{MESSAGE_MAX}
              </span>
            </div>
            <textarea
              maxLength={MESSAGE_MAX}
              placeholder="Write your entry..."
              value={newMessage}
              onChange={(e) => setNewMessage(e.target.value)}
              disabled={isSending}
              rows={3}
              className="w-full resize-none rounded-lg border border-border-low bg-card px-3 py-2 text-sm outline-none transition placeholder:text-muted focus:border-foreground/30 disabled:opacity-50"
            />
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleCreate}
              disabled={
                isSending || !newTitle.trim() || !newMessage.trim()
              }
              className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
            >
              {isSending ? "Creating..." : "Create"}
            </button>
            <button
              onClick={() => {
                setShowCreate(false);
                setNewTitle("");
                setNewMessage("");
              }}
              disabled={isSending}
              className="rounded-lg border border-border-low px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:opacity-50"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Entries List */}
      {isLoading ? (
        <div className="py-8 text-center text-sm text-muted">
          Loading entries...
        </div>
      ) : entries.length === 0 ? (
        <div className="rounded-xl border border-dashed border-border-low py-10 text-center">
          <p className="text-sm text-muted">No entries yet.</p>
          <p className="mt-1 text-xs text-muted/60">
            Create your first on-chain journal entry.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {entries.map((entry) => (
            <div
              key={entry.address}
              className="rounded-xl border border-border-low bg-cream/20 p-4 space-y-3"
            >
              {/* Entry Header */}
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <p className="truncate font-semibold">{entry.title}</p>
                  <a
                    href={getExplorerUrl(`/address/${entry.address}`)}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="font-mono text-xs text-muted underline underline-offset-2"
                  >
                    {ellipsify(entry.address, 6)}
                  </a>
                </div>
                {editingAddress !== entry.address && (
                  <div className="flex shrink-0 gap-2">
                    <button
                      onClick={() => {
                        setEditingAddress(entry.address);
                        setEditMessage(entry.message);
                        setDeletingAddress(null);
                      }}
                      disabled={isSending}
                      className="rounded-md border border-border-low px-3 py-1 text-xs font-medium transition hover:bg-cream disabled:opacity-50"
                    >
                      Edit
                    </button>
                    {deletingAddress === entry.address ? (
                      <div className="flex gap-1.5">
                        <button
                          onClick={() => handleDelete(entry)}
                          disabled={isSending}
                          className="rounded-md bg-red-500/10 px-3 py-1 text-xs font-medium text-red-600 transition hover:bg-red-500/20 disabled:opacity-50 dark:text-red-400"
                        >
                          {isSending ? "Deleting..." : "Confirm"}
                        </button>
                        <button
                          onClick={() => setDeletingAddress(null)}
                          disabled={isSending}
                          className="rounded-md border border-border-low px-3 py-1 text-xs font-medium transition hover:bg-cream disabled:opacity-50"
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => {
                          setDeletingAddress(entry.address);
                          setEditingAddress(null);
                        }}
                        disabled={isSending}
                        className="rounded-md border border-border-low px-3 py-1 text-xs font-medium text-red-500 transition hover:bg-red-500/10 disabled:opacity-50"
                      >
                        Delete
                      </button>
                    )}
                  </div>
                )}
              </div>

              {/* Message or Edit Form */}
              {editingAddress === entry.address ? (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <label className="text-xs text-muted">Message</label>
                    <span className="text-xs text-muted">
                      {editMessage.length}/{MESSAGE_MAX}
                    </span>
                  </div>
                  <textarea
                    maxLength={MESSAGE_MAX}
                    value={editMessage}
                    onChange={(e) => setEditMessage(e.target.value)}
                    disabled={isSending}
                    rows={3}
                    className="w-full resize-none rounded-lg border border-border-low bg-card px-3 py-2 text-sm outline-none transition focus:border-foreground/30 disabled:opacity-50"
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleUpdate(entry)}
                      disabled={isSending || !editMessage.trim()}
                      className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
                    >
                      {isSending ? "Saving..." : "Save"}
                    </button>
                    <button
                      onClick={() => setEditingAddress(null)}
                      disabled={isSending}
                      className="rounded-lg border border-border-low px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:opacity-50"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <p className="text-sm leading-relaxed text-foreground/80">
                  {entry.message}
                </p>
              )}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
