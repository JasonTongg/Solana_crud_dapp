"use client";

import useSWR from "swr";
import { type Address, type Base58EncodedBytes } from "@solana/kit";
import { useCluster } from "../../components/cluster-context";
import { useSolanaClient } from "../solana-client-context";
import { useWallet } from "../wallet/context";
import {
  CRUD_PROGRAM_ADDRESS,
  getJouralEntryStateDecoder,
  type JouralEntryState,
} from "../../generated/crud";

export type JournalEntry = JouralEntryState & { address: Address };

export function useJournalEntries() {
  const { wallet } = useWallet();
  const { cluster } = useCluster();
  const client = useSolanaClient();
  const owner = wallet?.account.address;

  const { data, isLoading, error, mutate } = useSWR(
    owner ? (["journal-entries", cluster, owner] as const) : null,
    async ([, , ownerAddr]) => {
      const result = await client.rpc
        .getProgramAccounts(CRUD_PROGRAM_ADDRESS, {
          encoding: "base64",
          withContext: true,
          filters: [
            {
              memcmp: {
                offset: 8n,
                bytes: ownerAddr as unknown as Base58EncodedBytes,
                encoding: "base58",
              },
            },
          ],
        })
        .send();

      const decoder = getJouralEntryStateDecoder();
      return result.value
        .map((item: { pubkey: Address; account: { data: unknown } }) => {
          try {
            const [base64Data] = item.account.data as [string, "base64"];
            const bytes = Uint8Array.from(atob(base64Data), (c) =>
              c.charCodeAt(0),
            );
            const state = decoder.decode(bytes);
            return { ...state, address: item.pubkey };
          } catch {
            return null;
          }
        })
        .filter((e: JournalEntry | null): e is JournalEntry => e !== null);
    },
    { revalidateOnFocus: true },
  );

  return {
    entries: data ?? [],
    isLoading,
    error,
    mutate,
  };
}
