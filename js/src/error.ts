export type MalevichErrorCode =
  | "UnequalChannels"
  | "NonRectangular"
  | "EmptyDimension"
  | "NonFiniteDomain"
  | "IncompatibleScale"
  | "InvalidParameter"
  | "DimensionTooLarge"
  | "AllocationFailed"
  | "Wasm";

export class MalevichError extends Error {
  readonly code: MalevichErrorCode;

  constructor(code: MalevichErrorCode, message: string) {
    super(message);
    this.name = "MalevichError";
    this.code = code;
  }
}

export function wrapWasm(error: unknown): never {
  const message = error instanceof Error ? error.message : String(error);
  throw new MalevichError("Wasm", message);
}
