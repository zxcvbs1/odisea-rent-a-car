import * as StellarSdk from "@stellar/stellar-sdk";
import type { ClientOptions } from "./types";

export async function buildContractClient<T = unknown>(options: ClientOptions): Promise<T> {
  const maybeContract = (StellarSdk as any)?.contract;
  const fromFn = maybeContract?.Client?.from as ((opts: ClientOptions) => Promise<unknown>) | undefined;
  if (!fromFn) {
    throw new Error(
      "Stellar SDK contract client is not available. Ensure correct SDK version or use a generated client."
    );
  }
  const client = await fromFn(options);
  return client as T;
}

