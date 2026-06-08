import {
  isSolanaError,
  SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM,
} from "@solana/kit";
import {
  getCrudErrorMessage,
  CRUD_ERROR__TITLE_TOO_LONG,
  CRUD_ERROR__MESSAGE_TOO_LONG,
  type CrudError,
} from "../generated/crud";

const CRUD_ERROR_CODES: Record<number, CrudError> = {
  [CRUD_ERROR__TITLE_TOO_LONG]: CRUD_ERROR__TITLE_TOO_LONG,
  [CRUD_ERROR__MESSAGE_TOO_LONG]: CRUD_ERROR__MESSAGE_TOO_LONG,
};

export function parseTransactionError(err: unknown): string {
  if (err instanceof Error && err.message.includes("User rejected")) {
    return "Transaction was rejected by the wallet.";
  }

  if (
    isSolanaError(err, SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM) &&
    typeof err.context?.code === "number"
  ) {
    const crudError = CRUD_ERROR_CODES[err.context.code];
    if (crudError !== undefined) {
      return getCrudErrorMessage(crudError);
    }
  }

  const message = getDeepestMessage(err);
  return message.length > 200 ? `${message.slice(0, 200)}...` : message;
}

function getDeepestMessage(err: unknown): string {
  let deepest = err instanceof Error ? err.message : String(err);
  let current: unknown = err;

  while (current instanceof Error && current.cause) {
    current = current.cause;
    if (current instanceof Error) {
      deepest = current.message;
    }
  }

  return deepest;
}
